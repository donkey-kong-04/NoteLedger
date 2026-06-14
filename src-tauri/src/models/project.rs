use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub parent_id: Option<i64>,
}
