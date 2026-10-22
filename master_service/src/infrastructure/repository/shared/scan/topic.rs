use pgdt::sqlx::{self, Row, postgres::PgRow};

use crate::domain::models::entity::Topic;

pub(crate) fn decode(row: &PgRow) -> Result<Topic, sqlx::Error> {
    Ok(Topic {
        id: row.try_get("id")?,
        slug: row.try_get("slug")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        polling_method_id: row.try_get("polling_method_id")?,
        selection_options: row.try_get("selection_options")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        deleted_at: row.try_get("deleted_at")?,
    })
}
