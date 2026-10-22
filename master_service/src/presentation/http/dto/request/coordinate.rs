use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinateQuery {
    pub longitude: f64,
    pub latitude: f64,
}
