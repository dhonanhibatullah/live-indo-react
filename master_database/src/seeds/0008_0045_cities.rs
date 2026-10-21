use crate::models::City;

use super::{EmbeddedSeed, LoadError, LoadedSeed};

include!(concat!(env!("OUT_DIR"), "/city_seeds.rs"));

/// Load versions 0008-0045 in order, decompressing only on iterator advancement.
pub fn load() -> impl ExactSizeIterator<Item = Result<LoadedSeed<City>, LoadError>> {
    FILES.iter().map(EmbeddedSeed::load)
}
