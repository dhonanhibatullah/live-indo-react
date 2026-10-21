use std::{
    collections::HashMap,
    env::{self, VarError},
    error::Error,
    fmt,
    fs::File,
    io::{self, Read},
    path::Path,
};

use crate::models::Env;

/// Load an optional .env in the working directory, overlaid by process variables.
/// Does not mutate the process environment or maintain global configuration.
pub fn load() -> Result<Env, LoadError> {
    load_from_path(".env")
}

/// As load(), with an explicit optional .env path for deployment or tooling.
pub fn load_from_path(path: impl AsRef<Path>) -> Result<Env, LoadError> {
    let values = read_dotenv(path.as_ref())?;
    from_sources(&values, |name| match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(LoadError::Variable {
            name,
            reason: "must contain valid Unicode",
        }),
    })
}

fn read_dotenv(path: &Path) -> Result<HashMap<String, String>, LoadError> {
    match File::open(path) {
        Ok(file) => parse_dotenv(file),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(source) => Err(LoadError::DotenvIo(source)),
    }
}

fn parse_dotenv(reader: impl Read) -> Result<HashMap<String, String>, LoadError> {
    dotenvy::from_read_iter(reader)
        .map(|entry| {
            entry.map_err(|error| match error {
                dotenvy::Error::Io(source) => LoadError::DotenvIo(source),
                // Parser errors may carry entire source lines, including secrets.
                _ => LoadError::DotenvSyntax,
            })
        })
        .collect()
}

fn from_sources(
    values: &HashMap<String, String>,
    process: impl Fn(&'static str) -> Result<Option<String>, LoadError>,
) -> Result<Env, LoadError> {
    let get = |name: &'static str, default: Option<&str>| {
        process(name)?
            .or_else(|| values.get(name).cloned())
            .or_else(|| default.map(str::to_owned))
            .ok_or(LoadError::Variable {
                name,
                reason: "must be explicitly supplied",
            })
    };
    let nonblank = |name: &'static str, value: String| {
        if value.trim().is_empty() {
            return Err(LoadError::Variable {
                name,
                reason: "must not be blank",
            });
        }
        Ok(value)
    };

    let port_name = "REACTION_DATABASE_POSTGRES_PORT";
    let postgres_port = get(port_name, Some("5432"))?
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or(LoadError::Variable {
            name: port_name,
            reason: "must be an integer from 1 to 65535",
        })?;

    let ssl_name = "REACTION_DATABASE_POSTGRES_SSL_MODE";
    let postgres_ssl_mode = get(ssl_name, Some("prefer"))?;
    if !matches!(
        postgres_ssl_mode.as_str(),
        "disable" | "allow" | "prefer" | "require" | "verify-ca" | "verify-full"
    ) {
        return Err(LoadError::Variable {
            name: ssl_name,
            reason: "must be disable, allow, prefer, require, verify-ca, or verify-full",
        });
    }

    Ok(Env {
        postgres_host: nonblank(
            "REACTION_DATABASE_POSTGRES_HOST",
            get("REACTION_DATABASE_POSTGRES_HOST", Some("127.0.0.1"))?,
        )?,
        postgres_port,
        postgres_username: nonblank(
            "REACTION_DATABASE_POSTGRES_USERNAME",
            get("REACTION_DATABASE_POSTGRES_USERNAME", None)?,
        )?,
        // Preserve passwords exactly, including intentional whitespace or emptiness.
        postgres_password: get("REACTION_DATABASE_POSTGRES_PASSWORD", None)?,
        postgres_database: nonblank(
            "REACTION_DATABASE_POSTGRES_DATABASE",
            get(
                "REACTION_DATABASE_POSTGRES_DATABASE",
                Some("reaction_database"),
            )?,
        )?,
        postgres_ssl_mode,
    })
}

/// Configuration errors never carry rejected values or raw .env source lines.
#[derive(Debug)]
pub enum LoadError {
    DotenvIo(io::Error),
    DotenvSyntax,
    Variable {
        name: &'static str,
        reason: &'static str,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DotenvIo(_) => formatter.write_str("could not read the environment file"),
            Self::DotenvSyntax => formatter.write_str("environment file contains an invalid entry"),
            Self::Variable { name, reason } => write!(formatter, "{name} {reason}"),
        }
    }
}

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DotenvIo(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/config_env.rs"]
mod tests;
