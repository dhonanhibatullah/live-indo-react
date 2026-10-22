use serde::Deserialize;

use crate::presentation::http::dto::boundary::GeoJsonBoundary;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateIndonesiaBoundaryRequest {
    pub boundary: Option<GeoJsonBoundary>,
    pub boundary_source: Option<String>,
    pub boundary_reference_date: Option<String>,
    #[serde(default)]
    pub clear_boundary_reference_date: bool,
}

impl TryFrom<UpdateIndonesiaBoundaryRequest>
    for crate::domain::usecases::management::indonesia_boundary::UpdateIndonesiaBoundary
{
    type Error = crate::presentation::http::dto::response::error::ApiError;

    fn try_from(value: UpdateIndonesiaBoundaryRequest) -> Result<Self, Self::Error> {
        Ok(Self {
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
        })
    }
}
