use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    pub id: i64,
    pub type_id: i64,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub closed_description: String,
    pub start_date: String,
    pub due_date: Option<String>,
    pub is_closed: bool,
    pub closed_date: Option<String>,
    pub project_id: i64,
    pub category1_ids: Vec<i64>,
    pub category2_ids: Vec<i64>,
    pub category3_ids: Vec<i64>,
    pub category4_ids: Vec<i64>,
}
