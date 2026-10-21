use std::fmt;

/// PostgreSQL connection settings. Loading and validation belong to config.
///
/// No environment variables are read here. Credentials have no built-in defaults
/// and this model deliberately does not implement serialization.
#[derive(Clone)]
pub struct Env {
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_username: String,
    pub postgres_password: String,
    pub postgres_database: String,
    /// PostgreSQL SSL mode, validated by the configuration loader.
    pub postgres_ssl_mode: String,
}

impl fmt::Debug for Env {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Env")
            .field("postgres_host", &self.postgres_host)
            .field("postgres_port", &self.postgres_port)
            .field("postgres_username", &self.postgres_username)
            .field("postgres_password", &"[REDACTED]")
            .field("postgres_database", &self.postgres_database)
            .field("postgres_ssl_mode", &self.postgres_ssl_mode)
            .finish()
    }
}
