use serde::{Deserialize, Serialize};

/// Seed input for a stable polling behavior; database IDs are resolved by code.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PollingMethod {
    pub code: String,
    pub name: String,
    pub description: String,
}
