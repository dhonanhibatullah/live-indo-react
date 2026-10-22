use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::Boundary;

/// National-boundary seed input for the singleton `indonesia_boundary` row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndonesiaBoundary {
    pub boundary: Boundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<NaiveDate>,
}
