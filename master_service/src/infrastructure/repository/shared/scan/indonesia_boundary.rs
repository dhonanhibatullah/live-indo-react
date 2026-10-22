use pgdt::sqlx::{Row, postgres::PgRow};

use crate::domain::models::entity::IndonesiaBoundary;

use super::super::{error::OperationError, geo};

pub(crate) fn decode(row: &PgRow) -> Result<IndonesiaBoundary, OperationError> {
    let boundary: String = row.try_get("boundary_geojson")?;
    Ok(IndonesiaBoundary {
        id: row.try_get("id")?,
        boundary: geo::decode_boundary(&boundary)?,
        boundary_source: row.try_get("boundary_source")?,
        boundary_reference_date: row.try_get("boundary_reference_date")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
