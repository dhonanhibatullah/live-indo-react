use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::domain::{
    models::entity::{City, GeoIp, IndonesiaBoundary, PollingMethod, Province, Topic},
    usecases::geographic::Region,
};

use super::super::{boundary::GeoJsonBoundary, city_kind::CityKindDto};

#[derive(Clone, Debug, Serialize)]
pub struct AuditResponse {
    pub created_at: String,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct TopicResponse {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub polling_method_id: i64,
    pub selection_options: Vec<String>,
    pub is_active: bool,
    pub audit: AuditResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct PollingMethodResponse {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: String,
    pub is_active: bool,
    pub audit: AuditResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct ProvinceResponse {
    pub id: i64,
    pub kemendagri_code: String,
    pub name: String,
    pub boundary: GeoJsonBoundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<String>,
    pub is_active: bool,
    pub audit: AuditResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct CityResponse {
    pub id: i64,
    pub province_id: i64,
    pub kemendagri_code: String,
    pub kind: CityKindDto,
    pub name: String,
    pub boundary: GeoJsonBoundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<String>,
    pub is_active: bool,
    pub audit: AuditResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct IndonesiaBoundaryResponse {
    pub id: i64,
    pub boundary: GeoJsonBoundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<String>,
    pub audit: AuditResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct RegionResponse {
    pub province: ProvinceResponse,
    pub city: Option<CityResponse>,
}

pub(crate) fn topic(value: Topic) -> TopicResponse {
    TopicResponse {
        id: value.id,
        slug: value.slug,
        name: value.name,
        description: value.description,
        polling_method_id: value.polling_method_id,
        selection_options: value.selection_options,
        is_active: value.is_active,
        audit: audit(value.created_at, value.updated_at, value.deleted_at),
    }
}

pub(crate) fn polling_method(value: PollingMethod) -> PollingMethodResponse {
    PollingMethodResponse {
        id: value.id,
        code: value.code,
        name: value.name,
        description: value.description,
        is_active: value.is_active,
        audit: audit(value.created_at, value.updated_at, value.deleted_at),
    }
}

pub(crate) fn province(value: Province) -> ProvinceResponse {
    ProvinceResponse {
        id: value.id,
        kemendagri_code: value.kemendagri_code,
        name: value.name,
        boundary: value.boundary.into(),
        boundary_source: value.boundary_source,
        boundary_reference_date: value.boundary_reference_date.map(date),
        is_active: value.is_active,
        audit: audit(value.created_at, value.updated_at, value.deleted_at),
    }
}

pub(crate) fn city(value: City) -> CityResponse {
    CityResponse {
        id: value.id,
        province_id: value.province_id,
        kemendagri_code: value.kemendagri_code,
        kind: value.kind.into(),
        name: value.name,
        boundary: value.boundary.into(),
        boundary_source: value.boundary_source,
        boundary_reference_date: value.boundary_reference_date.map(date),
        is_active: value.is_active,
        audit: audit(value.created_at, value.updated_at, value.deleted_at),
    }
}

pub(crate) fn indonesia_boundary(value: IndonesiaBoundary) -> IndonesiaBoundaryResponse {
    IndonesiaBoundaryResponse {
        id: value.id,
        boundary: value.boundary.into(),
        boundary_source: value.boundary_source,
        boundary_reference_date: value.boundary_reference_date.map(date),
        audit: audit(value.created_at, value.updated_at, None),
    }
}

pub(crate) fn region(value: Region) -> RegionResponse {
    RegionResponse {
        province: province(value.province),
        city: value.city.map(city),
    }
}

fn audit(
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
) -> AuditResponse {
    AuditResponse {
        created_at: timestamp(created_at),
        updated_at: updated_at.map(timestamp),
        deleted_at: deleted_at.map(timestamp),
    }
}

fn timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

fn date(value: NaiveDate) -> String {
    value.to_string()
}

/// Provider estimate; fields remain nullable and no lookup IP is echoed.
#[derive(Clone, Serialize)]
pub struct GeoIpResponse {
    pub continent_code: Option<String>,
    pub country_code: Option<String>,
    pub country_name: Option<String>,
    pub subdivisions: Vec<GeoIpSubdivisionResponse>,
    pub city_name: Option<String>,
    pub postal_code: Option<String>,
    pub coordinates: Option<GeoIpCoordinatesResponse>,
    pub accuracy_radius_km: Option<u16>,
    pub time_zone: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct GeoIpSubdivisionResponse {
    pub code: Option<String>,
    pub name: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct GeoIpCoordinatesResponse {
    pub longitude: f64,
    pub latitude: f64,
}

pub(crate) fn geoip(value: GeoIp) -> GeoIpResponse {
    GeoIpResponse {
        continent_code: value.continent_code,
        country_code: value.country_code,
        country_name: value.country_name,
        subdivisions: value
            .subdivisions
            .into_iter()
            .map(|subdivision| GeoIpSubdivisionResponse {
                code: subdivision.code,
                name: subdivision.name,
            })
            .collect(),
        city_name: value.city_name,
        postal_code: value.postal_code,
        coordinates: value
            .coordinates
            .map(|coordinate| GeoIpCoordinatesResponse {
                longitude: coordinate.longitude,
                latitude: coordinate.latitude,
            }),
        accuracy_radius_km: value.accuracy_radius_km,
        time_zone: value.time_zone,
    }
}
