use std::{str::FromStr, time::Duration};

use sqlx::{
    ConnectOptions, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};

use crate::models::Env;

use super::{DatabaseError, info, log_error};

fn options(env: &Env) -> Result<PgConnectOptions, DatabaseError> {
    if env.postgres_port == 0
        || env.postgres_host.trim().is_empty()
        || env.postgres_username.trim().is_empty()
        || env.postgres_database.trim().is_empty()
    {
        return Err(DatabaseError::invalid(
            "configure database connection",
            "host, username, database and a nonzero port are required",
        ));
    }
    let ssl = PgSslMode::from_str(&env.postgres_ssl_mode).map_err(|_| {
        DatabaseError::invalid(
            "configure database connection",
            "invalid PostgreSQL SSL mode",
        )
    })?;
    Ok(PgConnectOptions::new_without_pgpass()
        .host(&env.postgres_host)
        .port(env.postgres_port)
        .username(&env.postgres_username)
        .password(&env.postgres_password)
        .database(&env.postgres_database)
        .ssl_mode(ssl)
        .application_name(env!("CARGO_PKG_NAME"))
        .disable_statement_logging())
}

/// Connect to an already migrated reaction database. The caller owns pool shutdown.
pub async fn connect(env: &Env) -> Result<PgPool, DatabaseError> {
    info(format_args!("database connection starting"));
    let result = connect_inner(env).await;
    match &result {
        Ok(_) => info(format_args!(
            "database connected and schema prerequisites verified"
        )),
        Err(error) => log_error(error),
    }
    result
}

async fn connect_inner(env: &Env) -> Result<PgPool, DatabaseError> {
    let pool = open_pool(env).await?;
    if let Err(error) = check_schema(&pool).await {
        pool.close().await;
        return Err(error);
    }
    Ok(pool)
}

/// Connect before schema creation so the caller can migrate before checking it.
pub async fn connect_for_migrations(env: &Env) -> Result<PgPool, DatabaseError> {
    info(format_args!("migration database connection starting"));
    let result = open_pool(env).await;
    match &result {
        Ok(_) => info(format_args!("migration database connected")),
        Err(error) => log_error(error),
    }
    result
}

async fn open_pool(env: &Env) -> Result<PgPool, DatabaseError> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(options(env)?)
        .await
        .map_err(|error| DatabaseError::sql("connect database", error))
}

/// Verify schema prerequisites after migrations without opening another pool.
pub async fn verify_schema(pool: &PgPool) -> Result<(), DatabaseError> {
    let result = check_schema(pool).await;
    if let Err(error) = &result {
        log_error(error);
    }
    result
}

async fn check_schema(pool: &PgPool) -> Result<(), DatabaseError> {
    let (postgis, reactions, location_source): (bool, bool, bool) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'postgis'), to_regclass('reactions') IS NOT NULL, to_regtype('reaction_location_source') IS NOT NULL"
    ).fetch_one(pool).await.map_err(|error| DatabaseError::sql("check database schema", error))?;
    let missing: Vec<_> = [
        ("postgis", postgis),
        ("reactions", reactions),
        ("reaction_location_source", location_source),
    ]
    .into_iter()
    .filter_map(|(name, exists)| (!exists).then_some(name))
    .collect();
    if !missing.is_empty() {
        return Err(DatabaseError::invalid(
            "check database schema",
            format!(
                "missing {}; apply schema migrations first",
                missing.join(", ")
            ),
        ));
    }
    info(format_args!("database schema check succeeded"));
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/database_connection.rs"]
mod tests;
