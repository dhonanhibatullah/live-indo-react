use serde::{Deserialize, Serialize};

/// GeoJSON boundary matching the database's 2D MultiPolygon geometry type.
///
/// Coordinates are polygons -> rings -> [longitude, latitude] positions.
/// The enclosing seed file declares the SRID; importers must require EPSG:4326.
/// Ring closure, nonempty geometry, and topology are checked during import and
/// by PostGIS, not by deserialization alone.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Boundary {
    MultiPolygon {
        coordinates: Vec<Vec<Vec<[f64; 2]>>>,
    },
}
