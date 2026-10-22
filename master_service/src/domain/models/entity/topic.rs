use chrono::{DateTime, Utc};

/// A nationwide subject that reactions are grouped by.
#[derive(Clone, Debug, PartialEq)]
pub struct Topic {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub polling_method_id: i64,
    pub selection_options: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
