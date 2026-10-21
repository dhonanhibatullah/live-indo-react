use std::path::Path;

use sqlx::{PgPool, migrate::Migrator};

use super::{DatabaseError, info, log_error};

/// Apply pending up migrations using SQLx history, checksums, and database locking.
pub async fn run(pool: &PgPool, directory: &Path) -> Result<(), DatabaseError> {
    info(format_args!("schema migrations starting"));
    let result = run_inner(pool, directory).await;
    match &result {
        Ok(()) => info(format_args!("schema migrations complete")),
        Err(error) => log_error(error),
    }
    result
}

async fn run_inner(pool: &PgPool, directory: &Path) -> Result<(), DatabaseError> {
    let migrator = Migrator::new(directory)
        .await
        .map_err(|error| DatabaseError::new("load schema migrations", error))?;
    if migrator.iter().next().is_none() {
        return Err(DatabaseError::invalid(
            "load schema migrations",
            "migration directory contains no migrations",
        ));
    }
    migrator
        .run(pool)
        .await
        .map_err(|error| DatabaseError::new("apply schema migrations", error))
}
