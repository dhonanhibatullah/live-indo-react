use pgdt::sqlx::{self, Row, postgres::PgRow};

use crate::domain::models::entity::PollingMethod;

pub(crate) fn decode(row: &PgRow) -> Result<PollingMethod, sqlx::Error> {
    Ok(PollingMethod {
        id: row.try_get("id")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        deleted_at: row.try_get("deleted_at")?,
    })
}
