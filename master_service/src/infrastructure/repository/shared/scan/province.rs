use pgdt::sqlx::{Row, postgres::PgRow};

use crate::domain::models::entity::Province;

use super::super::{error::OperationError, geo};

pub(crate) fn decode(row: &PgRow) -> Result<Province, OperationError> {
    let boundary: String = row.try_get("boundary_geojson")?;
    Ok(Province {
        id: row.try_get("id")?,
        kemendagri_code: row.try_get("kemendagri_code")?,
        name: row.try_get("name")?,
        boundary: geo::decode_boundary(&boundary)?,
        boundary_source: row.try_get("boundary_source")?,
        boundary_reference_date: row.try_get("boundary_reference_date")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        deleted_at: row.try_get("deleted_at")?,
    })
}
