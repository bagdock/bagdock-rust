use reqwest::Client;
use serde_json::Value;
use crate::error::Result;
use crate::types::PaginatedResponse;

pub struct LoyaltyResource {
    http: Client,
    base_url: String,
}

impl LoyaltyResource {
    pub fn new(http: Client, base_url: String) -> Self {
        Self { http, base_url }
    }

    pub async fn list_members(&self) -> Result<PaginatedResponse<Value>> {
        let url = format!("{}/loyalty/members", self.base_url);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn award_points(&self, data: &Value) -> Result<Value> {
        let url = format!("{}/loyalty/points/award", self.base_url);
        let resp = self.http.post(&url).json(data).send().await?;
        Ok(resp.json().await?)
    }
}
