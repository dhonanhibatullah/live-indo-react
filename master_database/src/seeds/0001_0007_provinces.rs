use crate::models::Province;

use super::{EmbeddedSeed, LoadError, LoadedSeed};

include!(concat!(env!("OUT_DIR"), "/province_seeds.rs"));

/// Load versions 0001-0007 in order, decompressing only on iterator advancement.
pub fn load() -> impl ExactSizeIterator<Item = Result<LoadedSeed<Province>, LoadError>> {
    FILES.iter().map(EmbeddedSeed::load)
}
