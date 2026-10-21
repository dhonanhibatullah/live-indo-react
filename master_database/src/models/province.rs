use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::boundary::Boundary;

/// Province seed input. PostgreSQL generates IDs, activation, and audit defaults.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Province {
    pub kemendagri_code: String,
    pub name: String,
    pub boundary: Boundary,
    pub boundary_source: String,
    /// Effective date supplied by the dataset, not its generation/import time.
    pub boundary_reference_date: Option<NaiveDate>,
}
