use reqwest::Client;
use serde_json::Value;
use crate::error::Result;
use crate::types::PaginatedResponse;

pub struct OperatorResource {
    http: Client,
    base_url: String,
}

impl OperatorResource {
    pub fn new(http: Client, base_url: String) -> Self {
        Self { http, base_url }
    }

    pub async fn list_facilities(&self) -> Result<PaginatedResponse<Value>> {
        let url = format!("{}/operator/facilities", self.base_url);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn get_facility(&self, id: &str) -> Result<Value> {
        let url = format!("{}/operator/facilities/{}", self.base_url, id);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn list_contacts(&self) -> Result<PaginatedResponse<Value>> {
        let url = format!("{}/operator/contacts", self.base_url);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn create_contact(&self, data: &Value) -> Result<Value> {
        let url = format!("{}/operator/contacts", self.base_url);
        let resp = self.http.post(&url).json(data).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn list_units(&self) -> Result<PaginatedResponse<Value>> {
        let url = format!("{}/operator/units", self.base_url);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    pub async fn list_invoices(&self) -> Result<PaginatedResponse<Value>> {
        let url = format!("{}/operator/invoices", self.base_url);
        let resp = self.http.get(&url).send().await?;
        Ok(resp.json().await?)
    }
}
