use std::{error::Error as StdError, fmt};

pub type Position = [f64; 2];

/// A structurally valid 2D WGS84 MultiPolygon.
///
/// Coordinates are polygons -> rings -> [longitude, latitude] positions.
/// Topological validity is not checked by this type or the master schema.
#[derive(Clone, Debug, PartialEq)]
pub struct Boundary {
    coordinates: Vec<Vec<Vec<Position>>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundaryError {
    Empty,
    EmptyPolygon,
    RingTooShort,
    UnclosedRing,
    NonFiniteCoordinate,
    LongitudeOutOfRange,
    LatitudeOutOfRange,
}

impl Boundary {
    pub fn new(coordinates: Vec<Vec<Vec<Position>>>) -> Result<Self, BoundaryError> {
        if coordinates.is_empty() {
            return Err(BoundaryError::Empty);
        }

        for polygon in &coordinates {
            if polygon.is_empty() {
                return Err(BoundaryError::EmptyPolygon);
            }

            for ring in polygon {
                if ring.len() < 4 {
                    return Err(BoundaryError::RingTooShort);
                }

                if ring.first() != ring.last() {
                    return Err(BoundaryError::UnclosedRing);
                }

                for position in ring {
                    if !position[0].is_finite() || !position[1].is_finite() {
                        return Err(BoundaryError::NonFiniteCoordinate);
                    }

                    if !(-180.0..=180.0).contains(&position[0]) {
                        return Err(BoundaryError::LongitudeOutOfRange);
                    }

                    if !(-90.0..=90.0).contains(&position[1]) {
                        return Err(BoundaryError::LatitudeOutOfRange);
                    }
                }
            }
        }

        Ok(Self { coordinates })
    }

    pub fn coordinates(&self) -> &[Vec<Vec<Position>>] {
        &self.coordinates
    }
}

impl fmt::Display for BoundaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "boundary has no polygons",
            Self::EmptyPolygon => "boundary polygon has no rings",
            Self::RingTooShort => "boundary ring has fewer than four positions",
            Self::UnclosedRing => "boundary ring is not closed",
            Self::NonFiniteCoordinate => "boundary has a nonfinite coordinate",
            Self::LongitudeOutOfRange => "boundary longitude is outside -180..=180",
            Self::LatitudeOutOfRange => "boundary latitude is outside -90..=90",
        })
    }
}

impl StdError for BoundaryError {}
