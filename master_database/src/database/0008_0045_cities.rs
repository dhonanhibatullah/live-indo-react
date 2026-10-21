use chrono::NaiveDate;
use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    models::{City, CityKind},
    seeds::{self, EmbeddedSeed},
};

use super::{DatabaseError, Group, RowOutcome, SeedSummary};

const EXISTS: &str = "SELECT id FROM cities WHERE kemendagri_code = $1";
const PARENT: &str = "SELECT id FROM provinces WHERE kemendagri_code = $1";
const INSERT: &str = "INSERT INTO cities (province_id, kemendagri_code, kind, name, boundary, boundary_source, boundary_reference_date) VALUES ($1, $2, $3::city_kind, $4, ST_GeomFromGeoJSON($5::text), $6, $7) ON CONFLICT (kemendagri_code) DO NOTHING RETURNING id";

pub async fn seed(pool: &PgPool) -> Result<SeedSummary, DatabaseError> {
    super::seed_files(pool, seeds::cities::FILES, Group::Cities).await
}

pub(super) struct PreparedCity {
    code: String,
    province: String,
    kind: &'static str,
    name: String,
    boundary: String,
    source: String,
    date: Option<NaiveDate>,
}

fn validate_parent(city: &City) -> Result<(), DatabaseError> {
    if city
        .kemendagri_code
        .split_once('.')
        .map(|(prefix, _)| prefix)
        != Some(city.province_kemendagri_code.as_str())
    {
        return Err(DatabaseError::invalid(
            "validate city province code",
            "city code prefix must match the parent province code",
        )
        .record(&city.kemendagri_code));
    }
    Ok(())
}

pub(super) fn prepare(file: &'static EmbeddedSeed) -> Result<Vec<PreparedCity>, DatabaseError> {
    let loaded = file
        .load::<City>()
        .map_err(|error| DatabaseError::new("load city seed", error).file(file))?;
    loaded
        .data
        .records
        .into_iter()
        .map(|record| {
            validate_parent(&record).map_err(|error| error.file(file))?;
            let boundary = serde_json::to_string(&record.boundary).map_err(|error| {
                DatabaseError::new("serialize city boundary", error)
                    .file(file)
                    .record(&record.kemendagri_code)
            })?;
            Ok(PreparedCity {
                code: record.kemendagri_code,
                province: record.province_kemendagri_code,
                kind: match record.kind {
                    CityKind::City => "city",
                    CityKind::Regency => "regency",
                },
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
    row: PreparedCity,
) -> Result<RowOutcome, DatabaseError> {
    let existing: Option<i64> = sqlx::query_scalar(EXISTS)
        .bind(&row.code)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|error| DatabaseError::sql("check city existence", error).record(&row.code))?;
    let (id, inserted) = if let Some(id) = existing {
        (id, false)
    } else {
        let parent: Option<i64> = sqlx::query_scalar(PARENT)
            .bind(&row.province)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|error| {
                DatabaseError::sql("find city parent province", error).record(&row.code)
            })?;
        let parent = parent.ok_or_else(|| {
            DatabaseError::invalid(
                "find city parent province",
                "parent province is missing; apply province seeds first",
            )
            .record(&row.code)
        })?;
        let inserted: Option<i64> = sqlx::query_scalar(INSERT)
            .bind(parent)
            .bind(&row.code)
            .bind(row.kind)
            .bind(&row.name)
            .bind(&row.boundary)
            .bind(&row.source)
            .bind(row.date)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|error| DatabaseError::sql("insert city", error).record(&row.code))?;
        match inserted {
            Some(id) => (id, true),
            None => {
                let id = sqlx::query_scalar(EXISTS)
                    .bind(&row.code)
                    .fetch_one(&mut **tx)
                    .await
                    .map_err(|error| {
                        DatabaseError::sql("resolve concurrently inserted city", error)
                            .record(&row.code)
                    })?;
                (id, false)
            }
        }
    };
    Ok(RowOutcome {
        table: "cities",
        code: row.code,
        id,
        inserted,
    })
}

#[cfg(test)]
#[path = "../../tests/unit/database_cities.rs"]
mod tests;
