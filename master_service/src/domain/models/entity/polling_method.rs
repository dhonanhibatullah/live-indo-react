use chrono::{DateTime, Utc};

/// A stable polling behavior referenced by topics.
#[derive(Clone, Debug, PartialEq)]
pub struct PollingMethod {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
