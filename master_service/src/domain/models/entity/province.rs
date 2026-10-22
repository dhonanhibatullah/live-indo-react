use chrono::{DateTime, NaiveDate, Utc};

use super::boundary::Boundary;

/// An Indonesian province with its canonical WGS84 boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct Province {
    pub id: i64,
    pub kemendagri_code: String,
    pub name: String,
    pub boundary: Boundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}
