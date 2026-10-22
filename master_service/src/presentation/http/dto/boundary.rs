use serde::{Deserialize, Serialize};

use crate::domain::models::entity::{Boundary, BoundaryError};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum GeoJsonType {
    MultiPolygon,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GeoJsonBoundary {
    #[serde(rename = "type")]
    pub kind: GeoJsonType,
    pub coordinates: Vec<Vec<Vec<[f64; 2]>>>,
}

impl GeoJsonBoundary {
    pub(crate) fn into_boundary(self) -> Result<Boundary, BoundaryError> {
        if self.kind != GeoJsonType::MultiPolygon {
            return Err(BoundaryError::Empty);
        }
        Boundary::new(self.coordinates)
    }
}

impl From<Boundary> for GeoJsonBoundary {
    fn from(value: Boundary) -> Self {
        Self {
            kind: GeoJsonType::MultiPolygon,
            coordinates: value.coordinates().to_vec(),
        }
    }
}
