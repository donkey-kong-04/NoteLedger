use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub parent_id: Option<i64>,
    pub is_closed: bool,
    #[serde(default)]
    pub is_template: bool,
    /// Mandatory calendar date ('YYYY-MM-DD'); empty on insert means today.
    #[serde(default)]
    pub date: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub category1_ids: Vec<i64>,
    pub category2_ids: Vec<i64>,
    pub category3_ids: Vec<i64>,
    pub category4_ids: Vec<i64>,
}
