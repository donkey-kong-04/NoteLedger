use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PicklistType {
    Category1,
    Category2,
    Category3,
    Category4,
    LogType,
}

impl PicklistType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PicklistType::Category1 => "category_1",
            PicklistType::Category2 => "category_2",
            PicklistType::Category3 => "category_3",
            PicklistType::Category4 => "category_4",
            PicklistType::LogType => "log_type",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "category_1" => Some(PicklistType::Category1),
            "category_2" => Some(PicklistType::Category2),
            "category_3" => Some(PicklistType::Category3),
            "category_4" => Some(PicklistType::Category4),
            "log_type" => Some(PicklistType::LogType),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PicklistValue {
    pub id: i64,
    pub picklist_type: String,
    pub label: String,
}
