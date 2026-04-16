use reqwest::Client;
use serde_json::Value;
use crate::error::Result;
use crate::types::PaginatedResponse;

pub struct MarketplaceResource {
    http: Client,
    base_url: String,
}

impl MarketplaceResource {
    pub fn new(http: Client, base_url: String) -> Self {
        Self { http, base_url }
    }

    pub async fn search(&self) -> Result<PaginatedResponse<Value>> {
        let url = format!("{}/marketplace/search", self.base_url);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn get_listing(&self, id: &str) -> Result<Value> {
        let url = format!("{}/marketplace/listings/{}", self.base_url, id);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn create_rental(&self, data: &Value) -> Result<Value> {
        let url = format!("{}/marketplace/rentals", self.base_url);
        let resp = self.http.post(&url).json(data).send().await?;
        Ok(resp.json().await?)
    }
}
