use serde::Deserialize;

use crate::presentation::http::dto::{boundary::GeoJsonBoundary, city_kind::CityKindDto};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateCityRequest {
    pub province_id: i64,
    pub kemendagri_code: String,
    pub kind: CityKindDto,
    pub name: String,
    pub boundary: GeoJsonBoundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateCityRequest {
    pub kind: Option<CityKindDto>,
    pub name: Option<String>,
    pub boundary: Option<GeoJsonBoundary>,
    pub boundary_source: Option<String>,
    pub boundary_reference_date: Option<String>,
    #[serde(default)]
    pub clear_boundary_reference_date: bool,
    pub is_active: Option<bool>,
}

impl TryFrom<CreateCityRequest> for crate::domain::usecases::management::city::CreateCity {
    type Error = crate::presentation::http::dto::response::error::ApiError;

    fn try_from(value: CreateCityRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            province_id: value.province_id,
            kemendagri_code: value.kemendagri_code,
            kind: value.kind.into_domain(),
            name: value.name,
            boundary: value
                .boundary
                .into_boundary()
                .map_err(|_| Self::Error::bad_args("boundary is invalid"))?,
            boundary_source: value.boundary_source,
            boundary_reference_date: super::super::date::parse_date(value.boundary_reference_date)?,
            is_active: value.is_active,
        })
    }
}

impl TryFrom<UpdateCityRequest> for crate::domain::usecases::management::city::UpdateCity {
    type Error = crate::presentation::http::dto::response::error::ApiError;

    fn try_from(value: UpdateCityRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            id: 0,
            kind: value.kind.map(|kind| kind.into_domain()),
            name: value.name,
            boundary: value
                .boundary
                .map(|boundary| boundary.into_boundary())
                .transpose()
                .map_err(|_| Self::Error::bad_args("boundary is invalid"))?,
            boundary_source: value.boundary_source,
            boundary_reference_date: super::super::date::date_change(
                value.boundary_reference_date,
                value.clear_boundary_reference_date,
            )?,
            is_active: value.is_active,
        })
    }
}
