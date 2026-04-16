use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub object: String,
    pub data: Vec<T>,
    pub has_more: bool,
    pub total_count: Option<u64>,
}
