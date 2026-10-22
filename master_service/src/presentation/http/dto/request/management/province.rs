use serde::Deserialize;

use crate::presentation::http::dto::boundary::GeoJsonBoundary;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProvinceRequest {
    pub kemendagri_code: String,
    pub name: String,
    pub boundary: GeoJsonBoundary,
    pub boundary_source: String,
    pub boundary_reference_date: Option<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateProvinceRequest {
    pub name: Option<String>,
    pub boundary: Option<GeoJsonBoundary>,
    pub boundary_source: Option<String>,
    pub boundary_reference_date: Option<String>,
    #[serde(default)]
    pub clear_boundary_reference_date: bool,
    pub is_active: Option<bool>,
}

impl TryFrom<CreateProvinceRequest>
    for crate::domain::usecases::management::province::CreateProvince
{
    type Error = crate::presentation::http::dto::response::error::ApiError;

    fn try_from(value: CreateProvinceRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            kemendagri_code: value.kemendagri_code,
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

impl TryFrom<UpdateProvinceRequest>
    for crate::domain::usecases::management::province::UpdateProvince
{
    type Error = crate::presentation::http::dto::response::error::ApiError;

    fn try_from(value: UpdateProvinceRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            id: 0,
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
