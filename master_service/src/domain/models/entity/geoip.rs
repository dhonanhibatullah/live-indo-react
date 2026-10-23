use std::fmt;

/// Estimated IP location. Missing fields are unavailable, not empty or zero.
/// Region codes belong to the provider and are not Kemendagri identifiers.
#[derive(Clone, PartialEq)]
pub struct GeoIp {
    pub continent_code: Option<String>,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    /// Broadest subdivision first, matching the provider hierarchy.
    pub subdivisions: Vec<GeoIpSubdivision>,
    pub city_name: Option<String>,
    pub postal_code: Option<String>,
    pub coordinates: Option<GeoIpCoordinates>,
    pub accuracy_radius_km: Option<u16>,
    pub time_zone: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeoIpSubdivision {
    pub code: Option<String>,
    pub name: Option<String>,
}

/// Approximate WGS84 location, never proof of a user's physical position.
#[derive(Clone, Copy, PartialEq)]
pub struct GeoIpCoordinates {
    pub longitude: f64,
    pub latitude: f64,
}

// Do not let routine diagnostics dump location data.
impl fmt::Debug for GeoIp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("GeoIp").finish_non_exhaustive()
    }
}

impl fmt::Debug for GeoIpCoordinates {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GeoIpCoordinates")
            .finish_non_exhaustive()
    }
}
