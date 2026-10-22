use crate::domain::models::{
    AppContext,
    entity::{City, Province},
};

use super::super::UsecaseFuture;

/// A province plus the city/regency containing the coordinate, when one exists.
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    pub province: Province,
    pub city: Option<City>,
}

/// Resolve a WGS84 coordinate to the smallest known administrative region.
pub trait GetRegionByCoord: Send + Sync {
    fn get_region_by_coord<'a>(
        &'a self,
        context: &'a AppContext,
        longitude: f64,
        latitude: f64,
    ) -> UsecaseFuture<'a, Option<Region>>;
}
