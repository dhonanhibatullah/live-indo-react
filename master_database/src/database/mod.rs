//! PostgreSQL migration and seed operations.
//!
//! Connect with `connection::connect_for_migrations`, run `migrations::run`,
//! verify the schema, then `seed` and close the pool. `connection::connect` is
//! available for callers with an already migrated schema. Each seed file commits
//! independently; a failed run resumes from recorded history.

#[path = "0008_0045_cities.rs"]
pub mod cities;
pub mod connection;
#[path = "0047_indonesia_boundary.rs"]
pub mod indonesia_boundary;
pub mod migrations;
#[path = "0046_polling_methods.rs"]
pub mod polling_methods;
#[path = "0001_0007_provinces.rs"]
pub mod provinces;

use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

use sqlx::{PgPool, Postgres, Transaction};

use crate::seeds::EmbeddedSeed;

const SEED_LOCK: i64 = 0x4D41535445525344;
const HISTORY: &str = "SELECT version, filename, checksum_sha256 FROM seed_history WHERE version = $1 OR filename = $2";
const RECORD_HISTORY: &str =
    "INSERT INTO seed_history (version, filename, checksum_sha256) VALUES ($1, $2, $3)";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SeedSummary {
    pub files_applied: usize,
    pub files_skipped: usize,
    pub rows_inserted: usize,
    pub rows_skipped: usize,
}

impl SeedSummary {
    fn add(&mut self, other: Self) {
        self.files_applied += other.files_applied;
        self.files_skipped += other.files_skipped;
        self.rows_inserted += other.rows_inserted;
        self.rows_skipped += other.rows_skipped;
    }
}

/// Apply geographic, polling-method, then national-boundary seeds, stopping at the first failure.
pub async fn seed(pool: &PgPool) -> Result<SeedSummary, DatabaseError> {
    info(format_args!("seed run started"));
    let mut summary = provinces::seed(pool).await?;
    summary.add(cities::seed(pool).await?);
    summary.add(polling_methods::seed(pool).await?);
    summary.add(indonesia_boundary::seed(pool).await?);
    info(format_args!(
        "seed run complete files_applied={} files_skipped={} rows_inserted={} rows_skipped={}",
        summary.files_applied, summary.files_skipped, summary.rows_inserted, summary.rows_skipped
    ));
    Ok(summary)
}

#[derive(Clone, Copy)]
enum Group {
    Provinces,
    Cities,
    PollingMethods,
    IndonesiaBoundary,
}

enum PreparedRows {
    Provinces(Vec<provinces::PreparedProvince>),
    Cities(Vec<cities::PreparedCity>),
    PollingMethods(Vec<crate::models::PollingMethod>),
    IndonesiaBoundary(Vec<indonesia_boundary::PreparedIndonesiaBoundary>),
}

struct RowOutcome {
    table: &'static str,
    code: String,
    id: i64,
    inserted: bool,
}

async fn seed_files(
    pool: &PgPool,
    files: &'static [EmbeddedSeed],
    group: Group,
) -> Result<SeedSummary, DatabaseError> {
    let mut summary = SeedSummary::default();
    for file in files {
        info(format_args!(
            "file started version={} filename={}",
            file.version, file.filename
        ));
        match apply_file(pool, file, group).await {
            Ok(outcome) => summary.add(outcome),
            Err(error) => {
                log_error(&error);
                return Err(error);
            }
        }
    }
    Ok(summary)
}

async fn apply_file(
    pool: &PgPool,
    file: &'static EmbeddedSeed,
    group: Group,
) -> Result<SeedSummary, DatabaseError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|error| DatabaseError::sql("begin seed transaction", error).file(file))?;
    let result = apply_pending(&mut tx, file, group).await;
    match result {
        Ok(None) => {
            tx.commit()
                .await
                .map_err(|error| DatabaseError::sql("finish history check", error).file(file))?;
            info(format_args!(
                "file skipped version={} filename={} reason=already-applied",
                file.version, file.filename
            ));
            Ok(SeedSummary {
                files_skipped: 1,
                ..Default::default()
            })
        }
        Ok(Some(rows)) => {
            // A commit error can have an uncertain outcome: never claim success or retry blindly.
            tx.commit()
                .await
                .map_err(|error| DatabaseError::sql("commit seed transaction", error).file(file))?;
            let mut summary = SeedSummary {
                files_applied: 1,
                ..Default::default()
            };
            for row in rows {
                let action = if row.inserted {
                    summary.rows_inserted += 1;
                    "inserted"
                } else {
                    summary.rows_skipped += 1;
                    "skipped-existing"
                };
                info(format_args!(
                    "row {action} version={} filename={} table={} code={} id={}",
                    file.version, file.filename, row.table, row.code, row.id
                ));
            }
            info(format_args!(
                "file committed version={} filename={} inserted={} skipped={}",
                file.version, file.filename, summary.rows_inserted, summary.rows_skipped
            ));
            Ok(summary)
        }
        Err(error) => {
            if let Err(rollback) = tx.rollback().await {
                log_error(&DatabaseError::sql("rollback seed transaction", rollback).file(file));
            }
            Err(error)
        }
    }
}

async fn apply_pending(
    tx: &mut Transaction<'_, Postgres>,
    file: &'static EmbeddedSeed,
    group: Group,
) -> Result<Option<Vec<RowOutcome>>, DatabaseError> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(SEED_LOCK)
        .execute(&mut **tx)
        .await
        .map_err(|error| DatabaseError::sql("lock seed transaction", error).file(file))?;
    let history: Vec<(i64, String, String)> = sqlx::query_as(HISTORY)
        .bind(file.version)
        .bind(file.filename)
        .fetch_all(&mut **tx)
        .await
        .map_err(|error| DatabaseError::sql("check seed history", error).file(file))?;
    if !history.is_empty() {
        if history.len() == 1
            && history[0].0 == file.version
            && history[0].1 == file.filename
            && history[0].2 == file.checksum_sha256
        {
            return Ok(None);
        }
        return Err(DatabaseError::invalid(
            "check seed history",
            "recorded filename/version/checksum conflicts with the embedded seed",
        )
        .file(file));
    }

    let prepared = tokio::task::spawn_blocking(move || match group {
        Group::Provinces => provinces::prepare(file).map(PreparedRows::Provinces),
        Group::Cities => cities::prepare(file).map(PreparedRows::Cities),
        Group::PollingMethods => polling_methods::prepare(file).map(PreparedRows::PollingMethods),
        Group::IndonesiaBoundary => {
            indonesia_boundary::prepare(file).map(PreparedRows::IndonesiaBoundary)
        }
    })
    .await
    .map_err(|error| DatabaseError::new("prepare seed task", error).file(file))??;
    let mut outcomes = Vec::new();
    match prepared {
        PreparedRows::Provinces(rows) => {
            for row in rows {
                outcomes.push(
                    provinces::execute(tx, row)
                        .await
                        .map_err(|error| error.file(file))?,
                );
            }
        }
        PreparedRows::Cities(rows) => {
            for row in rows {
                outcomes.push(
                    cities::execute(tx, row)
                        .await
                        .map_err(|error| error.file(file))?,
                );
            }
        }
        PreparedRows::PollingMethods(rows) => {
            for row in rows {
                outcomes.push(
                    polling_methods::execute(tx, row)
                        .await
                        .map_err(|error| error.file(file))?,
                );
            }
        }
        PreparedRows::IndonesiaBoundary(rows) => {
            for row in rows {
                outcomes.push(
                    indonesia_boundary::execute(tx, row)
                        .await
                        .map_err(|error| error.file(file))?,
                );
            }
        }
    }
    sqlx::query(RECORD_HISTORY)
        .bind(file.version)
        .bind(file.filename)
        .bind(file.checksum_sha256)
        .execute(&mut **tx)
        .await
        .map_err(|error| DatabaseError::sql("record seed history", error).file(file))?;
    Ok(Some(outcomes))
}

/// Display/Debug are safe for console output; source() retains native diagnostics.
pub struct DatabaseError {
    pub operation: &'static str,
    pub version: Option<i64>,
    pub filename: Option<&'static str>,
    pub code: Option<String>,
    details: Box<ErrorDetails>,
}

struct ErrorDetails {
    sqlstate: Option<String>,
    constraint: Option<String>,
    message: String,
    source: Box<dyn Error + Send + Sync>,
}

impl DatabaseError {
    pub fn sqlstate(&self) -> Option<&str> {
        self.details.sqlstate.as_deref()
    }

    pub fn constraint(&self) -> Option<&str> {
        self.details.constraint.as_deref()
    }

    fn new(operation: &'static str, source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            operation,
            version: None,
            filename: None,
            code: None,
            details: Box::new(ErrorDetails {
                sqlstate: None,
                constraint: None,
                message: "operation failed".into(),
                source: Box::new(source),
            }),
        }
    }
    fn sql(operation: &'static str, source: sqlx::Error) -> Self {
        let sqlstate = source
            .as_database_error()
            .and_then(|error| error.code())
            .map(|code| code.into_owned());
        let constraint = source
            .as_database_error()
            .and_then(|error| error.constraint())
            .map(str::to_owned);
        let mut error = Self::new(operation, source);
        error.details.sqlstate = sqlstate;
        error.details.constraint = constraint;
        error
    }
    fn invalid(operation: &'static str, message: impl Into<String>) -> Self {
        let message = message.into();
        let mut error = Self::new(operation, io::Error::other(message.clone()));
        error.details.message = message;
        error
    }
    fn file(mut self, file: &'static EmbeddedSeed) -> Self {
        self.version = Some(file.version);
        self.filename = Some(file.filename);
        self
    }
    fn record(mut self, code: &str) -> Self {
        self.code = Some(code.to_owned());
        self
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.operation, self.details.message)?;
        if let Some(filename) = self.filename {
            write!(formatter, " filename={filename}")?;
        }
        if let Some(code) = &self.code {
            write!(formatter, " code={code}")?;
        }
        if let Some(state) = &self.details.sqlstate {
            write!(formatter, " sqlstate={state}")?;
        }
        if let Some(constraint) = &self.details.constraint {
            write!(formatter, " constraint={constraint}")?;
        }
        Ok(())
    }
}
impl fmt::Debug for DatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}
impl Error for DatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.details.source.as_ref())
    }
}

fn write_log(writer: &mut impl Write, level: &str, message: fmt::Arguments<'_>) {
    let _ = writeln!(writer, "[{level}] {message}");
}
fn info(message: fmt::Arguments<'_>) {
    write_log(&mut io::stdout().lock(), "INFO", message);
}
fn log_error(error: &DatabaseError) {
    write_log(&mut io::stderr().lock(), "ERROR", format_args!("{error}"));
}

#[cfg(test)]
#[path = "../../tests/unit/database.rs"]
mod tests;
