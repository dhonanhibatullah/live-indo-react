use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Shared envelope for the ordered province and city JSON seed files.
///
/// The importer validates version/name against the filename and uses the raw
/// file bytes for the seed-history checksum, not a reserialized instance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeedFile<T> {
    pub version: i64,
    pub name: String,
    pub srid: u32,
    pub source: SeedSource,
    pub records: Vec<T>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeedSource {
    pub repository: String,
    pub revision: String,
    pub url: String,
    /// Hash of the upstream download, distinct from the local seed-file checksum.
    pub sha256: String,
    pub generated_at: DateTime<Utc>,
    pub data_reference: String,
    pub publisher: SeedPublisher,
    pub copyright: SeedCopyright,
    pub license_file: String,
    pub normalization: String,
    pub repaired_region_codes: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SeedPublisher {
    pub name: String,
    pub url: String,
    pub github: String,
    pub data_reference: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SeedCopyright {
    pub text: String,
    pub license: String,
    pub license_url: String,
}

/// Locally maintained reference catalog without geographic source metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CatalogSeedFile<T> {
    pub version: i64,
    pub name: String,
    pub records: Vec<T>,
}
