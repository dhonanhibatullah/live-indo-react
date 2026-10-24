use std::{error::Error, fmt, io, str::FromStr, sync::Arc};

use pgdt::sqlx::{
    ConnectOptions, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions, PgSslMode},
};

use crate::domain::{contracts::utility::Logger, models::AppEnv};
use crate::infrastructure::utility::logger::WriterLogger;

pub fn logger(env: &AppEnv) -> Arc<dyn Logger> {
    Arc::new(WriterLogger::new(
        io::stdout(),
        env.logger_format,
        env.logger_level,
    ))
}

pub struct Drivers {
    pub logger: Arc<dyn Logger>,
    pub database: pgdt::Pgdt,
}

pub async fn build(env: &AppEnv, logger: Arc<dyn Logger>) -> Result<Drivers, DriverError> {
    let pool = connect(env).await?;
    Ok(Drivers {
        logger,
        database: pgdt::Pgdt::new(pool),
    })
}

async fn connect(env: &AppEnv) -> Result<PgPool, DriverError> {
    let postgres_ssl_mode =
        PgSslMode::from_str(&env.postgres_ssl_mode).map_err(|_| DriverError::InvalidSslMode)?;

    let options = PgConnectOptions::new_without_pgpass()
        .host(&env.postgres_host)
        .port(env.postgres_port)
        .username(&env.postgres_username)
        .password(&env.postgres_password)
        .database(&env.postgres_database)
        .ssl_mode(postgres_ssl_mode)
        .application_name(env!("CARGO_PKG_NAME"))
        .disable_statement_logging();

    PgPoolOptions::new()
        .max_connections(env.postgres_max_connections)
        .acquire_timeout(env.postgres_connect_timeout)
        .connect_with(options)
        .await
        .map_err(DriverError::Database)
}

#[derive(Debug)]
pub enum DriverError {
    InvalidSslMode,
    Database(pgdt::sqlx::Error),
}

impl fmt::Display for DriverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSslMode => formatter.write_str("PostgreSQL SSL mode is invalid"),
            Self::Database(_) => formatter.write_str("could not initialize the PostgreSQL pool"),
        }
    }
}

impl Error for DriverError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidSslMode => None,
            Self::Database(source) => Some(source),
        }
    }
}
