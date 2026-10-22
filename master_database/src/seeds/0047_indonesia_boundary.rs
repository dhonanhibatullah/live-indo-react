use crate::models::IndonesiaBoundary;

use super::{EmbeddedSeed, LoadError, LoadedSeed};

include!(concat!(env!("OUT_DIR"), "/indonesia_boundary_seeds.rs"));

/// Load the national-boundary seed; it contains one record.
pub fn load() -> impl ExactSizeIterator<Item = Result<LoadedSeed<IndonesiaBoundary>, LoadError>> {
    FILES.iter().map(EmbeddedSeed::load)
}
