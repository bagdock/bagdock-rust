use std::sync::Arc;
use reqwest::{Client, header};
use serde::{de::DeserializeOwned, Serialize};
use tokio::sync::Mutex;

use crate::error::{BagdockError, Result};
use crate::oauth::{OAuthEndpoints, TokenManager, SharedTokenManager};
use crate::resources::operator::OperatorResource;
use crate::resources::marketplace::MarketplaceResource;
use crate::resources::loyalty::LoyaltyResource;

const DEFAULT_BASE_URL: &str = "https://api.bagdock.com/api/v1";
const DEFAULT_MAX_RETRIES: u32 = 3;
const MAX_RETRY_CAP: u32 = 5;

enum AuthMode {
    Static(String),
    ClientCredentials(SharedTokenManager),
}

pub struct Bagdock {
    http: Client,
    base_url: String,
    auth: AuthMode,
    pub operator: OperatorResource,
    pub marketplace: MarketplaceResource,
    pub loyalty: LoyaltyResource,
}

impl Bagdock {
    /// Create a client authenticated with an API key.
    pub fn new(api_key: &str) -> Result<Self> {
        Self::build(AuthMode::Static(api_key.to_string()), DEFAULT_BASE_URL)
    }

    /// Create a client with an existing OAuth access token.
    pub fn with_access_token(access_token: &str) -> Result<Self> {
        Self::build(AuthMode::Static(access_token.to_string()), DEFAULT_BASE_URL)
    }

    /// Create a client using OAuth2 client credentials (auto-fetches tokens).
    pub fn with_client_credentials(
        client_id: &str,
        client_secret: &str,
        scopes: Vec<String>,
        endpoints: Option<&OAuthEndpoints>,
    ) -> Result<Self> {
        let tm = Arc::new(Mutex::new(TokenManager::new(client_id, client_secret, scopes, endpoints)));
        Self::build(AuthMode::ClientCredentials(tm), DEFAULT_BASE_URL)
    }

    /// Create a client with a custom base URL.
    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self> {
        Self::build(AuthMode::Static(api_key.to_string()), base_url)
    }

    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.max_retries = n.min(MAX_RETRY_CAP);
        self
    }

    fn build(auth: AuthMode, base_url: &str) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("bagdock-rust/0.1.0"));

        let http = Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| BagdockError::Config(e.to_string()))?;

        let base = base_url.trim_end_matches('/').to_string();

        Ok(Self {
            operator: OperatorResource::new(http.clone(), base.clone()),
            marketplace: MarketplaceResource::new(http.clone(), base.clone()),
            loyalty: LoyaltyResource::new(http.clone(), base.clone()),
            http,
            base_url: base,
            auth,
            max_retries: DEFAULT_MAX_RETRIES,
        })
    }

    async fn resolve_token(&self) -> Result<String> {
        match &self.auth {
            AuthMode::Static(token) => Ok(token.clone()),
            AuthMode::ClientCredentials(tm) => {
                let mut guard = tm.lock().await;
                guard.get_token().await
            }
        }
    }

    async fn invalidate_token(&self) {
        if let AuthMode::ClientCredentials(tm) = &self.auth {
            let mut guard = tm.lock().await;
            guard.invalidate();
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.resolve_token().await?;
        let resp = self.http.get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send().await?;

        if resp.status().as_u16() == 401 {
            if let AuthMode::ClientCredentials(_) = &self.auth {
                self.invalidate_token().await;
                let new_token = self.resolve_token().await?;
                let resp2 = self.http.get(&url)
                    .header("Authorization", format!("Bearer {}", new_token))
                    .send().await?;
                return handle_response(resp2).await;
            }
        }

        handle_response(resp).await
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.resolve_token().await?;
        let resp = self.http.post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(body).send().await?;

        if resp.status().as_u16() == 401 {
            if let AuthMode::ClientCredentials(_) = &self.auth {
                self.invalidate_token().await;
                let new_token = self.resolve_token().await?;
                let resp2 = self.http.post(&url)
                    .header("Authorization", format!("Bearer {}", new_token))
                    .json(body).send().await?;
                return handle_response(resp2).await;
            }
        }

        handle_response(resp).await
    }
}

async fn handle_response<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
    let status = resp.status();
    if status.is_client_error() || status.is_server_error() {
        let body: serde_json::Value = resp.json().await.unwrap_or_default();
        return Err(BagdockError::Api {
            status: status.as_u16(),
            code: body["code"].as_str().unwrap_or("unknown").to_string(),
            message: body["message"].as_str().unwrap_or("Request failed").to_string(),
        });
    }
    Ok(resp.json().await?)
}
