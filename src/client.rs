use reqwest::{Client, header};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{BagdockError, Result};
use crate::resources::operator::OperatorResource;
use crate::resources::marketplace::MarketplaceResource;
use crate::resources::loyalty::LoyaltyResource;

const DEFAULT_BASE_URL: &str = "https://api.bagdock.com/api/v1";

pub struct Bagdock {
    http: Client,
    base_url: String,
    pub operator: OperatorResource,
    pub marketplace: MarketplaceResource,
    pub loyalty: LoyaltyResource,
}

impl Bagdock {
    pub fn new(api_key: &str) -> Result<Self> {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> Result<Self> {
        if api_key.is_empty() {
            return Err(BagdockError::Authentication("Missing API key".into()));
        }

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                .map_err(|e| BagdockError::Config(e.to_string()))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("bagdock-rust/0.1.0"),
        );

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
        })
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.http.get(&url).send().await?;
        handle_response(resp).await
    }

    pub async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self.http.post(&url).json(body).send().await?;
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
            message: body["message"]
                .as_str()
                .unwrap_or("Request failed")
                .to_string(),
        });
    }
    Ok(resp.json().await?)
}
