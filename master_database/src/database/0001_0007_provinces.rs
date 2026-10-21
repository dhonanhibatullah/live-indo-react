use chrono::NaiveDate;
use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    models::Province,
    seeds::{self, EmbeddedSeed},
};

use super::{DatabaseError, Group, RowOutcome, SeedSummary};

const EXISTS: &str = "SELECT id FROM provinces WHERE kemendagri_code = $1";
const INSERT: &str = "INSERT INTO provinces (kemendagri_code, name, boundary, boundary_source, boundary_reference_date) VALUES ($1, $2, ST_GeomFromGeoJSON($3::text), $4, $5) ON CONFLICT (kemendagri_code) DO NOTHING RETURNING id";

pub async fn seed(pool: &PgPool) -> Result<SeedSummary, DatabaseError> {
    super::seed_files(pool, seeds::provinces::FILES, Group::Provinces).await
}

pub(super) struct PreparedProvince {
    code: String,
    name: String,
    boundary: String,
    source: String,
    date: Option<NaiveDate>,
}

pub(super) fn prepare(file: &'static EmbeddedSeed) -> Result<Vec<PreparedProvince>, DatabaseError> {
    let loaded = file
        .load::<Province>()
        .map_err(|error| DatabaseError::new("load province seed", error).file(file))?;
    loaded
        .data
        .records
        .into_iter()
        .map(|record| {
            let boundary = serde_json::to_string(&record.boundary).map_err(|error| {
                DatabaseError::new("serialize province boundary", error)
                    .file(file)
                    .record(&record.kemendagri_code)
            })?;
            Ok(PreparedProvince {
                code: record.kemendagri_code,
                name: record.name,
                boundary,
                source: record.boundary_source,
                date: record.boundary_reference_date,
            })
        })
        .collect()
}

pub(super) async fn execute(
    tx: &mut Transaction<'_, Postgres>,
    row: PreparedProvince,
) -> Result<RowOutcome, DatabaseError> {
    let existing: Option<i64> = sqlx::query_scalar(EXISTS)
        .bind(&row.code)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| DatabaseError::sql("check province existence", error).record(&row.code))?;
    let (id, inserted) = if let Some(id) = existing {
        (id, false)
    } else {
        let inserted: Option<i64> = sqlx::query_scalar(INSERT)
            .bind(&row.code)
            .bind(&row.name)
            .bind(&row.boundary)
            .bind(&row.source)
            .bind(row.date)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|error| DatabaseError::sql("insert province", error).record(&row.code))?;
        match inserted {
            Some(id) => (id, true),
            None => {
                let id = sqlx::query_scalar(EXISTS)
                    .bind(&row.code)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(|error| {
                        DatabaseError::sql("resolve concurrently inserted province", error)
                            .record(&row.code)
                    })?;
                (id, false)
            }
        }
    };
    Ok(RowOutcome {
        table: "provinces",
        code: row.code,
        id,
        inserted,
    })
}
