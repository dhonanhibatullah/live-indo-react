use std::time::Duration;

use crate::domain::models::logger::{Format, Level};

/// Logger and PostgreSQL settings for future composition and infrastructure.
/// Loading, defaults and additional runtime settings are defined separately.
///
/// Secrets are intentionally excluded from `Debug`; do not derive it on this type.
pub struct AppEnv {
    pub logger_format: Format,
    pub logger_level: Level,

    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_username: String,
    pub postgres_password: String,
    pub postgres_database: String,
    pub postgres_ssl_mode: String,
    pub postgres_max_connections: u32,
    pub postgres_connect_timeout: Duration,
}
