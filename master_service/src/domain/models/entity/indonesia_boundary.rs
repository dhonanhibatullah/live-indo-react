use chrono::{DateTime, NaiveDate, Utc};

use super::Boundary;

/// The singleton Indonesian national boundary, including waters outside provinces.
#[derive(Clone, Debug, PartialEq)]
pub struct IndonesiaBoundary {
    pub id: i64,
    pub boundary: Boundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}
