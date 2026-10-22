use chrono::NaiveDate;
use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    models::IndonesiaBoundary,
    seeds::{self, EmbeddedSeed},
};

use super::{DatabaseError, Group, RowOutcome, SeedSummary};

const EXISTS: &str = "SELECT id::bigint FROM indonesia_boundary WHERE id = 1";
const INSERT: &str = "INSERT INTO indonesia_boundary (id, boundary, boundary_source, boundary_reference_date) VALUES (1, ST_SetSRID(ST_GeomFromGeoJSON($1::text), 4326)::geometry(MultiPolygon,4326), $2, $3) ON CONFLICT (id) DO NOTHING RETURNING id::bigint";

pub async fn seed(pool: &PgPool) -> Result<SeedSummary, DatabaseError> {
    super::seed_files(
        pool,
        seeds::indonesia_boundary::FILES,
        Group::IndonesiaBoundary,
    )
    .await
}

pub(super) struct PreparedIndonesiaBoundary {
    boundary: String,
    source: String,
    date: Option<NaiveDate>,
}

pub(super) fn prepare(
    file: &'static EmbeddedSeed,
) -> Result<Vec<PreparedIndonesiaBoundary>, DatabaseError> {
    let loaded = file
        .load::<IndonesiaBoundary>()
        .map_err(|error| DatabaseError::new("load Indonesia boundary seed", error).file(file))?;
    loaded
        .data
        .records
        .into_iter()
        .map(|record| {
            let boundary = serde_json::to_string(&record.boundary).map_err(|error| {
                DatabaseError::new("serialize Indonesia boundary", error)
                    .file(file)
                    .record("indonesia")
            })?;
            Ok(PreparedIndonesiaBoundary {
                boundary,
                source: record.boundary_source,
                date: record.boundary_reference_date,
            })
        })
        .collect()
}

pub(super) async fn execute(
    tx: &mut Transaction<'_, Postgres>,
    row: PreparedIndonesiaBoundary,
) -> Result<RowOutcome, DatabaseError> {
    let existing: Option<i64> = sqlx::query_scalar(EXISTS)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| {
            DatabaseError::sql("check Indonesia boundary existence", error).record("indonesia")
        })?;
    let (id, inserted) = if let Some(id) = existing {
        (id, false)
    } else {
        let inserted: Option<i64> = sqlx::query_scalar(INSERT)
            .bind(&row.boundary)
            .bind(&row.source)
            .bind(row.date)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|error| {
                DatabaseError::sql("insert Indonesia boundary", error).record("indonesia")
            })?;
        match inserted {
            Some(id) => (id, true),
            None => {
                let id = sqlx::query_scalar(EXISTS)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(|error| {
                        DatabaseError::sql(
                            "resolve concurrently inserted Indonesia boundary",
                            error,
                        )
                        .record("indonesia")
                    })?;
                (id, false)
            }
        }
    };
    Ok(RowOutcome {
        table: "indonesia_boundary",
        code: "indonesia".into(),
        id,
        inserted,
    })
}
