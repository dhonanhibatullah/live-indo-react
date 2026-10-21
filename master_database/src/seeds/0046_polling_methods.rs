use crate::models::PollingMethod;

use super::{EmbeddedSeed, LoadError, LoadedCatalogSeed};

include!(concat!(env!("OUT_DIR"), "/polling_method_seeds.rs"));

/// Load the reference catalog lazily using the same embedded-byte checksum scheme.
pub fn load() -> impl ExactSizeIterator<Item = Result<LoadedCatalogSeed<PollingMethod>, LoadError>>
{
    FILES.iter().map(EmbeddedSeed::load_catalog)
}
