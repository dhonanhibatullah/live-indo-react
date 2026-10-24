use live_indo_proto::live_indo_react::master::v1 as proto;

use crate::domain::{
    models::entity::{City, CityKind, GeoIp, IndonesiaBoundary, PollingMethod, Province, Topic},
    usecases::geographic::Region,
};

use super::{
    audit::{audit, date},
    geometry::multi_polygon,
};

pub(crate) fn topic(value: Topic) -> proto::Topic {
    proto::Topic {
        id: value.id,
        slug: value.slug,
        name: value.name,
        description: value.description,
        polling_method_id: value.polling_method_id,
        selection_options: value.selection_options,
        is_active: value.is_active,
        audit: Some(audit(value.created_at, value.updated_at, value.deleted_at)),
    }
}

pub(crate) fn polling_method(value: PollingMethod) -> proto::PollingMethod {
    proto::PollingMethod {
        id: value.id,
        code: value.code,
        name: value.name,
        description: value.description,
        is_active: value.is_active,
        audit: Some(audit(value.created_at, value.updated_at, value.deleted_at)),
    }
}

pub(crate) fn province(value: Province) -> proto::Province {
    proto::Province {
        id: value.id,
        kemendagri_code: value.kemendagri_code,
        name: value.name,
        boundary: Some(multi_polygon(value.boundary)),
        boundary_source: value.boundary_source,
        boundary_reference_date: value.boundary_reference_date.map(date),
        is_active: value.is_active,
        audit: Some(audit(value.created_at, value.updated_at, value.deleted_at)),
    }
}

pub(crate) fn city(value: City) -> proto::City {
    proto::City {
        id: value.id,
        province_id: value.province_id,
        kemendagri_code: value.kemendagri_code,
        kind: city_kind(value.kind) as i32,
        name: value.name,
        boundary: Some(multi_polygon(value.boundary)),
        boundary_source: value.boundary_source,
        boundary_reference_date: value.boundary_reference_date.map(date),
        is_active: value.is_active,
        audit: Some(audit(value.created_at, value.updated_at, value.deleted_at)),
    }
}

pub(crate) fn indonesia_boundary(value: IndonesiaBoundary) -> proto::IndonesiaBoundary {
    proto::IndonesiaBoundary {
        id: value.id,
        boundary: Some(multi_polygon(value.boundary)),
        boundary_source: value.boundary_source,
        boundary_reference_date: value.boundary_reference_date.map(date),
        audit: Some(audit(value.created_at, value.updated_at, None)),
    }
}

pub(crate) fn region(value: Region) -> proto::Region {
    proto::Region {
        province: Some(province(value.province)),
        city: value.city.map(city),
    }
}

fn city_kind(value: CityKind) -> proto::CityKind {
    match value {
        CityKind::City => proto::CityKind::City,
        CityKind::Regency => proto::CityKind::Regency,
    }
}

pub(crate) fn geoip(value: GeoIp) -> proto::GeoIp {
    proto::GeoIp {
        continent_code: value.continent_code,
        country_code: value.country_code,
        country_name: value.country_name,
        subdivisions: value
            .subdivisions
            .into_iter()
            .map(|subdivision| proto::GeoIpSubdivision {
                code: subdivision.code,
                name: subdivision.name,
            })
            .collect(),
        city_name: value.city_name,
        postal_code: value.postal_code,
        coordinates: value.coordinates.map(|coordinate| proto::Coordinate {
            longitude: coordinate.longitude,
            latitude: coordinate.latitude,
        }),
        accuracy_radius_km: value.accuracy_radius_km.map(u32::from),
        time_zone: value.time_zone,
    }
}
