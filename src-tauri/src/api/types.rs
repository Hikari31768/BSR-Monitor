use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "itemsPerPage", default)]
    pub items_per_page: i32,
    #[serde(default)]
    pub page: i32,
    #[serde(default)]
    pub total: i32,
}