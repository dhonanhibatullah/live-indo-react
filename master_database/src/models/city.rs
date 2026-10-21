use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::boundary::Boundary;

/// City/regency seed input; resolve the parent code into a province ID at import.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct City {
    pub kemendagri_code: String,
    pub name: String,
    pub province_kemendagri_code: String,
    pub kind: CityKind,
    pub boundary: Boundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<NaiveDate>,
}

/// Values match the PostgreSQL city_kind enum and JSON seed spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CityKind {
    City,
    Regency,
}
