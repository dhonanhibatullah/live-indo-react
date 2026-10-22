use chrono::{DateTime, NaiveDate, Utc};

use super::boundary::Boundary;

/// A second-level city (kota) or regency (kabupaten).
#[derive(Clone, Debug, PartialEq)]
pub struct City {
    pub id: i64,
    pub province_id: i64,
    pub kemendagri_code: String,
    pub kind: CityKind,
    pub name: String,
    pub boundary: Boundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CityKind {
    City,
    Regency,
}

impl CityKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::City => "city",
            Self::Regency => "regency",
        }
    }
}
