use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLink {
    pub id: i64,
    pub project_id: i64,
    pub label: String,
    pub url: String,
}
