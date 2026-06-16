use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub category1_label: String,
    pub category2_label: String,
    pub category3_label: String,
    pub category4_label: String,
    pub dark_mode: bool,
    pub density: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            category1_label: "Category 1".to_string(),
            category2_label: "Category 2".to_string(),
            category3_label: "Category 3".to_string(),
            category4_label: "Category 4".to_string(),
            dark_mode: false,
            density: "normal".to_string(),
        }
    }
}
