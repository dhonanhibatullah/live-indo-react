use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    models::PollingMethod,
    seeds::{self, EmbeddedSeed},
};

use super::{DatabaseError, Group, RowOutcome, SeedSummary};

const EXISTS: &str = "SELECT id FROM polling_methods WHERE code = $1";
const INSERT: &str = "INSERT INTO polling_methods (code, name, description) VALUES ($1, $2, $3) ON CONFLICT (code) DO NOTHING RETURNING id";

pub async fn seed(pool: &PgPool) -> Result<SeedSummary, DatabaseError> {
    super::seed_files(pool, seeds::polling_methods::FILES, Group::PollingMethods).await
}

pub(super) fn prepare(file: &'static EmbeddedSeed) -> Result<Vec<PollingMethod>, DatabaseError> {
    file.load_catalog::<PollingMethod>()
        .map(|loaded| loaded.data.records)
        .map_err(|error| DatabaseError::new("load polling method seed", error).file(file))
}

pub(super) async fn execute(
    tx: &mut Transaction<'_, Postgres>,
    row: PollingMethod,
) -> Result<RowOutcome, DatabaseError> {
    let existing: Option<i64> = sqlx::query_scalar(EXISTS)
        .bind(&row.code)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| {
            DatabaseError::sql("check polling method existence", error).record(&row.code)
        })?;
    let (id, inserted) = if let Some(id) = existing {
        (id, false)
    } else {
        let inserted: Option<i64> = sqlx::query_scalar(INSERT)
            .bind(&row.code)
            .bind(&row.name)
            .bind(&row.description)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|error| {
                DatabaseError::sql("insert polling method", error).record(&row.code)
            })?;
        match inserted {
            Some(id) => (id, true),
            None => {
                let id = sqlx::query_scalar(EXISTS)
                    .bind(&row.code)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(|error| {
                        DatabaseError::sql("resolve concurrently inserted polling method", error)
                            .record(&row.code)
                    })?;
                (id, false)
            }
        }
    };
    Ok(RowOutcome {
        table: "polling_methods",
        code: row.code,
        id,
        inserted,
    })
}
