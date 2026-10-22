//! Build-time compressed seeds; no source JSON files are read at runtime.
//!
//! Each group loads lazily, one file per iterator step. Process and drop each
//! LoadedSeed before advancing to keep memory bounded to one seed file.
//!
//! ```no_run
//! # fn run() -> Result<(), master_database::seeds::LoadError> {
//! for seed in master_database::seeds::provinces::load() {
//!     let seed = seed?;
//!     // seed.file contains version, filename, and the original file checksum.
//!     // seed.data contains the typed seed envelope and province records.
//!     println!("{}: {} records", seed.file.filename, seed.data.records.len());
//! }
//! # Ok(())
//! # }
//! ```

#[path = "0008_0045_cities.rs"]
pub mod cities;
#[path = "0047_indonesia_boundary.rs"]
pub mod indonesia_boundary;
#[path = "0046_polling_methods.rs"]
pub mod polling_methods;
#[path = "0001_0007_provinces.rs"]
pub mod provinces;

mod error;

use std::{fmt, io::Read};

use flate2::read::GzDecoder;
use serde::de::DeserializeOwned;

use crate::models::{CatalogSeedFile, SeedFile};

pub use error::LoadError;

/// Metadata and compressed bytes generated from one original JSON seed file.
pub struct EmbeddedSeed {
    pub version: i64,
    pub filename: &'static str,
    /// SHA-256 of the original prettified JSON bytes, not the gzip payload.
    pub checksum_sha256: &'static str,
    compressed: &'static [u8],
}

impl EmbeddedSeed {
    /// Decompress the original JSON string without reading the filesystem.
    pub fn json(&self) -> Result<String, LoadError> {
        let mut json = String::new();
        GzDecoder::new(self.compressed)
            .read_to_string(&mut json)
            .map_err(|source| LoadError::Decompress {
                filename: self.filename,
                source,
            })?;
        Ok(json)
    }

    /// Decompress and deserialize just this seed. The temporary JSON is dropped.
    pub fn load<T: DeserializeOwned>(&'static self) -> Result<LoadedSeed<T>, LoadError> {
        let data = self.decode()?;
        Ok(LoadedSeed { file: self, data })
    }

    pub fn load_catalog<T: DeserializeOwned>(
        &'static self,
    ) -> Result<LoadedCatalogSeed<T>, LoadError> {
        let data = self.decode()?;
        Ok(LoadedCatalogSeed { file: self, data })
    }

    fn decode<T: DeserializeOwned>(&self) -> Result<T, LoadError> {
        let json = self.json()?;
        serde_json::from_str(&json).map_err(|source| LoadError::Deserialize {
            filename: self.filename,
            source,
        })
    }

    pub fn compressed_len(&self) -> usize {
        self.compressed.len()
    }
}

impl fmt::Debug for EmbeddedSeed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EmbeddedSeed")
            .field("version", &self.version)
            .field("filename", &self.filename)
            .field("checksum_sha256", &self.checksum_sha256)
            .field("compressed_len", &self.compressed_len())
            .finish()
    }
}

#[derive(Debug)]
pub struct LoadedSeed<T> {
    pub file: &'static EmbeddedSeed,
    pub data: SeedFile<T>,
}

#[cfg(test)]
#[path = "../../tests/unit/seeds.rs"]
mod tests;

#[derive(Debug)]
pub struct LoadedCatalogSeed<T> {
    pub file: &'static EmbeddedSeed,
    pub data: CatalogSeedFile<T>,
}
