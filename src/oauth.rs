use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use rand::Rng;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

use crate::error::{BagdockError, Result};

const DEFAULT_ISSUER: &str = "https://api.bagdock.com";
const DEVICE_CODE_GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct OAuthEndpoints {
    pub issuer: String,
    pub token_endpoint: Option<String>,
    pub authorize_endpoint: Option<String>,
    pub device_authorize_endpoint: Option<String>,
    pub revoke_endpoint: Option<String>,
    pub introspect_endpoint: Option<String>,
}

impl Default for OAuthEndpoints {
    fn default() -> Self {
        Self {
            issuer: DEFAULT_ISSUER.to_string(),
            token_endpoint: None,
            authorize_endpoint: None,
            device_authorize_endpoint: None,
            revoke_endpoint: None,
            introspect_endpoint: None,
        }
    }
}

impl OAuthEndpoints {
    fn resolve(&self, suffix: &str, override_url: &Option<String>) -> String {
        override_url.clone().unwrap_or_else(|| {
            format!("{}{}", self.issuer.trim_end_matches('/'), suffix)
        })
    }

    pub fn token(&self) -> String { self.resolve("/oauth2/token", &self.token_endpoint) }
    pub fn authorize(&self) -> String { self.resolve("/oauth2/authorize", &self.authorize_endpoint) }
    pub fn device_authorize(&self) -> String { self.resolve("/oauth2/device/authorize", &self.device_authorize_endpoint) }
    pub fn revoke(&self) -> String { self.resolve("/oauth2/token/revoke", &self.revoke_endpoint) }
    pub fn introspect(&self) -> String { self.resolve("/oauth2/token/introspect", &self.introspect_endpoint) }
}

#[derive(Debug, Clone)]
pub struct PKCEPair {
    pub code_verifier: String,
    pub code_challenge: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_uri_complete: Option<String>,
    pub expires_in: i64,
    pub interval: i64,
}

#[derive(Debug, Deserialize)]
struct OAuthErrorResponse {
    error: Option<String>,
    error_description: Option<String>,
}

// ---------------------------------------------------------------------------
// PKCE (RFC 7636)
// ---------------------------------------------------------------------------

pub fn generate_pkce() -> PKCEPair {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let code_verifier = URL_SAFE_NO_PAD.encode(&bytes);

    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let digest = hasher.finalize();
    let code_challenge = URL_SAFE_NO_PAD.encode(digest);

    PKCEPair { code_verifier, code_challenge }
}

// ---------------------------------------------------------------------------
// Authorize URL
// ---------------------------------------------------------------------------

pub fn build_authorize_url(
    client_id: &str,
    redirect_uri: &str,
    code_challenge: &str,
    scope: Option<&str>,
    state: Option<&str>,
    endpoints: Option<&OAuthEndpoints>,
) -> String {
    let ep = endpoints.cloned().unwrap_or_default();
    let mut params = vec![
        ("client_id", client_id.to_string()),
        ("redirect_uri", redirect_uri.to_string()),
        ("response_type", "code".to_string()),
        ("code_challenge", code_challenge.to_string()),
        ("code_challenge_method", "S256".to_string()),
    ];
    if let Some(s) = scope { params.push(("scope", s.to_string())); }
    if let Some(s) = state { params.push(("state", s.to_string())); }

    let query = serde_urlencoded::to_string(&params).unwrap_or_default();
    format!("{}?{}", ep.authorize(), query)
}

// ---------------------------------------------------------------------------
// Token helpers
// ---------------------------------------------------------------------------

async fn post_form(url: &str, data: &[(&str, &str)]) -> Result<HashMap<String, serde_json::Value>> {
    let client = Client::new();
    let resp = client.post(url).form(data).send().await?;
    let status = resp.status().as_u16();
    let body: HashMap<String, serde_json::Value> = resp.json().await.unwrap_or_default();

    if status >= 400 {
        let err_code = body.get("error").and_then(|v| v.as_str()).unwrap_or("oauth_error").to_string();
        let err_msg = body.get("error_description")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("HTTP {}", status))
            .to_string();
        return Err(BagdockError::Api { status, code: err_code, message: err_msg });
    }

    Ok(body)
}

pub async fn exchange_code(
    client_id: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
    client_secret: Option<&str>,
    endpoints: Option<&OAuthEndpoints>,
) -> Result<TokenResponse> {
    let ep = endpoints.cloned().unwrap_or_default();
    let mut data = vec![
        ("grant_type", "authorization_code"),
        ("client_id", client_id),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier),
    ];
    if let Some(s) = client_secret { data.push(("client_secret", s)); }

    let body = post_form(&ep.token(), &data).await?;
    serde_json::from_value(serde_json::Value::Object(body.into_iter().collect()))
        .map_err(|e| BagdockError::Config(e.to_string()))
}

pub async fn refresh_token(
    client_id: &str,
    refresh_token_value: &str,
    client_secret: Option<&str>,
    endpoints: Option<&OAuthEndpoints>,
) -> Result<TokenResponse> {
    let ep = endpoints.cloned().unwrap_or_default();
    let mut data = vec![
        ("grant_type", "refresh_token"),
        ("client_id", client_id),
        ("refresh_token", refresh_token_value),
    ];
    if let Some(s) = client_secret { data.push(("client_secret", s)); }

    let body = post_form(&ep.token(), &data).await?;
    serde_json::from_value(serde_json::Value::Object(body.into_iter().collect()))
        .map_err(|e| BagdockError::Config(e.to_string()))
}

pub async fn revoke_token(
    token: &str,
    token_type_hint: Option<&str>,
    endpoints: Option<&OAuthEndpoints>,
) -> Result<()> {
    let ep = endpoints.cloned().unwrap_or_default();
    let mut data = vec![("token", token)];
    if let Some(h) = token_type_hint { data.push(("token_type_hint", h)); }
    post_form(&ep.revoke(), &data).await?;
    Ok(())
}

pub async fn device_authorize(
    client_id: &str,
    scope: Option<&str>,
    endpoints: Option<&OAuthEndpoints>,
) -> Result<DeviceAuthResponse> {
    let ep = endpoints.cloned().unwrap_or_default();
    let mut data = vec![("client_id", client_id)];
    if let Some(s) = scope { data.push(("scope", s)); }

    let body = post_form(&ep.device_authorize(), &data).await?;
    serde_json::from_value(serde_json::Value::Object(body.into_iter().collect()))
        .map_err(|e| BagdockError::Config(e.to_string()))
}

pub async fn poll_device_token(
    client_id: &str,
    device_code: &str,
    interval_secs: u64,
    timeout_secs: u64,
    endpoints: Option<&OAuthEndpoints>,
) -> Result<TokenResponse> {
    let ep = endpoints.cloned().unwrap_or_default();
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let mut poll_interval = Duration::from_secs(interval_secs);

    while Instant::now() < deadline {
        tokio::time::sleep(poll_interval).await;
        let data = vec![
            ("grant_type", DEVICE_CODE_GRANT_TYPE),
            ("client_id", client_id),
            ("device_code", device_code),
        ];
        match post_form(&ep.token(), &data).await {
            Ok(body) => {
                return serde_json::from_value(serde_json::Value::Object(body.into_iter().collect()))
                    .map_err(|e| BagdockError::Config(e.to_string()));
            }
            Err(BagdockError::Api { code, .. }) if code == "authorization_pending" => continue,
            Err(BagdockError::Api { code, .. }) if code == "slow_down" => {
                poll_interval += Duration::from_secs(5);
                continue;
            }
            Err(e) => return Err(e),
        }
    }

    Err(BagdockError::Api {
        status: 408,
        code: "expired_token".to_string(),
        message: "Device authorization timed out".to_string(),
    })
}

// ---------------------------------------------------------------------------
// Token manager for client credentials (internal)
// ---------------------------------------------------------------------------

pub(crate) struct TokenManager {
    client_id: String,
    client_secret: String,
    scopes: Vec<String>,
    token_url: String,
    access_token: Option<String>,
    expires_at: Instant,
}

impl TokenManager {
    pub fn new(client_id: &str, client_secret: &str, scopes: Vec<String>, endpoints: Option<&OAuthEndpoints>) -> Self {
        let ep = endpoints.cloned().unwrap_or_default();
        Self {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
            scopes,
            token_url: ep.token(),
            access_token: None,
            expires_at: Instant::now(),
        }
    }

    pub async fn get_token(&mut self) -> Result<String> {
        if let Some(ref token) = self.access_token {
            if Instant::now() < self.expires_at {
                return Ok(token.clone());
            }
        }
        self.fetch_token().await
    }

    pub fn invalidate(&mut self) {
        self.access_token = None;
        self.expires_at = Instant::now();
    }

    async fn fetch_token(&mut self) -> Result<String> {
        let scope_str = self.scopes.join(" ");
        let mut data: Vec<(&str, &str)> = vec![
            ("grant_type", "client_credentials"),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];
        if !self.scopes.is_empty() {
            data.push(("scope", &scope_str));
        }

        let body = post_form(&self.token_url, &data).await?;
        let token = body.get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BagdockError::Config("Missing access_token in response".to_string()))?
            .to_string();
        let expires_in = body.get("expires_in")
            .and_then(|v| v.as_i64())
            .unwrap_or(3600);

        self.access_token = Some(token.clone());
        self.expires_at = Instant::now() + Duration::from_secs((expires_in - 60).max(0) as u64);
        Ok(token)
    }
}

pub(crate) type SharedTokenManager = Arc<Mutex<TokenManager>>;
