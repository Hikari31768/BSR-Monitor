use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "itemsPerPage")]
    pub items_per_page: i32,
    pub page: i32,
    pub total: i32,
}