use std::{error::Error as StdError, fmt};

use serde_json::{Value, json};

use crate::domain::models::{
    RepositoryError,
    entity::{Boundary, BoundaryError},
};

pub(crate) fn encode_boundary(boundary: &Boundary) -> Result<String, RepositoryError> {
    serde_json::to_string(&json!({
        "type": "MultiPolygon",
        "coordinates": boundary.coordinates(),
    }))
    .map_err(|source| RepositoryError::Failure {
        source: Box::new(source),
    })
}

pub(crate) fn decode_boundary(value: &str) -> Result<Boundary, RepositoryError> {
    let value: Value = serde_json::from_str(value).map_err(BoundaryDecodeError::from)?;
    if value.get("type").and_then(Value::as_str) != Some("MultiPolygon") {
        return Err(BoundaryDecodeError::WrongType.into());
    }
    let coordinates = value
        .get("coordinates")
        .cloned()
        .ok_or(BoundaryDecodeError::MissingCoordinates)?;
    let coordinates: Vec<Vec<Vec<[f64; 2]>>> =
        serde_json::from_value(coordinates).map_err(BoundaryDecodeError::from)?;
    let boundary = Boundary::new(coordinates).map_err(BoundaryDecodeError::from)?;
    Ok(boundary)
}

impl From<BoundaryDecodeError> for RepositoryError {
    fn from(source: BoundaryDecodeError) -> Self {
        Self::Failure {
            source: Box::new(source),
        }
    }
}

#[derive(Debug)]
enum BoundaryDecodeError {
    Json(serde_json::Error),
    WrongType,
    MissingCoordinates,
    Boundary(BoundaryError),
}

impl From<serde_json::Error> for BoundaryDecodeError {
    fn from(source: serde_json::Error) -> Self {
        Self::Json(source)
    }
}

impl From<BoundaryError> for BoundaryDecodeError {
    fn from(source: BoundaryError) -> Self {
        Self::Boundary(source)
    }
}

impl fmt::Display for BoundaryDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Json(_) => "stored boundary is not valid GeoJSON",
            Self::WrongType => "stored boundary is not a MultiPolygon",
            Self::MissingCoordinates => "stored boundary has no coordinates",
            Self::Boundary(_) => "stored boundary is structurally invalid",
        })
    }
}

impl StdError for BoundaryDecodeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Json(source) => Some(source),
            Self::Boundary(source) => Some(source),
            Self::WrongType | Self::MissingCoordinates => None,
        }
    }
}
