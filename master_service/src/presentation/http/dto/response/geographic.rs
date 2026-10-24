use serde::Serialize;

pub use super::model::{CityResponse, ProvinceResponse, RegionResponse};

#[derive(Clone, Debug, Serialize)]
pub struct RegionLookupResponse {
    pub region: Option<RegionResponse>,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct InsideResponse {
    pub inside: bool,
}

#[derive(Clone, Serialize)]
pub struct RegionByIpResponse {
    pub geoip: Option<super::model::GeoIpResponse>,
    pub region: Option<RegionResponse>,
}
