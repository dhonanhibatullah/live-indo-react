use std::net::IpAddr;

use crate::domain::models::{
    AppContext,
    entity::{City, GeoIp, Province},
};

use super::super::UsecaseFuture;

/// A province plus the city/regency containing the coordinate, when one exists.
#[derive(Clone, Debug, PartialEq)]
pub struct Region {
    pub province: Province,
    pub city: Option<City>,
}

/// Resolve coordinates or an estimated IP location to our administrative catalog.
pub trait GetRegion: Send + Sync {
    /// Retain the provider estimate even when coordinates or region matches are
    /// absent. No record yields both fields None. Provider failures remain errors.
    /// This lookup does not enforce country admission or infer regions from names.
    fn get_region_by_ip<'a>(
        &'a self,
        context: &'a AppContext,
        ip: IpAddr,
    ) -> UsecaseFuture<'a, RegionByIp>;

    /// Resolve a WGS84 coordinate to the smallest known administrative region.
    fn get_region_by_coord<'a>(
        &'a self,
        context: &'a AppContext,
        longitude: f64,
        latitude: f64,
    ) -> UsecaseFuture<'a, Option<Region>>;
}

/// Provider location and our canonical region match are independently optional.
#[derive(Clone, Debug, PartialEq)]
pub struct RegionByIp {
    pub geoip: Option<GeoIp>,
    pub region: Option<Region>,
}
