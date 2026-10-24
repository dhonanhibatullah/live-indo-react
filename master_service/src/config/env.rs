use std::{
    collections::HashMap,
    env::{self, VarError},
    error::Error,
    fmt,
    fs::File,
    io::{self, Read},
    net::SocketAddr,
    path::Path,
    str::FromStr,
    time::Duration,
};

use crate::domain::models::{
    AppEnv,
    logger::{Format, Level},
};

const POSTGRES_HOST: &str = "MASTER_SERVICE_POSTGRES_HOST";
const POSTGRES_PORT: &str = "MASTER_SERVICE_POSTGRES_PORT";
const POSTGRES_USERNAME: &str = "MASTER_SERVICE_POSTGRES_USERNAME";
const POSTGRES_PASSWORD: &str = "MASTER_SERVICE_POSTGRES_PASSWORD";
const POSTGRES_DATABASE: &str = "MASTER_SERVICE_POSTGRES_DATABASE";
const POSTGRES_SSL_MODE: &str = "MASTER_SERVICE_POSTGRES_SSL_MODE";
const POSTGRES_MAX_CONNECTIONS: &str = "MASTER_SERVICE_POSTGRES_MAX_CONNECTIONS";
const POSTGRES_CONNECT_TIMEOUT: &str = "MASTER_SERVICE_POSTGRES_CONNECT_TIMEOUT_MS";
const CACHE_MAX_CAPACITY: &str = "MASTER_SERVICE_CACHE_MAX_CAPACITY";
const CACHE_INDIVIDUAL_TTL: &str = "MASTER_SERVICE_CACHE_INDIVIDUAL_TTL_MS";
const CACHE_LIST_TTL: &str = "MASTER_SERVICE_CACHE_LIST_TTL_MS";
const LOGGER_FORMAT: &str = "MASTER_SERVICE_LOGGER_FORMAT";
const LOGGER_LEVEL: &str = "MASTER_SERVICE_LOGGER_LEVEL";
const HTTP_ADDR: &str = "MASTER_SERVICE_HTTP_ADDR";
const GRPC_ADDR: &str = "MASTER_SERVICE_GRPC_ADDR";
const GEOIP_CONF_PATH: &str = "MASTER_SERVICE_GEOIP_CONF_PATH";
const GEOIP_DATABASE_PATH: &str = "MASTER_SERVICE_GEOIP_DATABASE_PATH";
const SHUTDOWN_TIMEOUT: &str = "MASTER_SERVICE_SHUTDOWN_TIMEOUT_MS";

/// Load an optional `.env` from the working directory, overlaid by process variables.
/// Does not mutate the process environment.
pub fn load() -> Result<AppEnv, LoadError> {
    load_from_path(".env")
}

/// Same as `load`, with an explicit optional `.env` path for tooling or deployment.
pub fn load_from_path(path: impl AsRef<Path>) -> Result<AppEnv, LoadError> {
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
) -> Result<AppEnv, LoadError> {
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
    let positive = |name: &'static str, value: String, reason: &'static str| {
        value
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or(LoadError::Variable { name, reason })
    };

    let logger_format = get(LOGGER_FORMAT, Some("plain"))?;
    let logger_format = Format::from_str(&logger_format).map_err(|_| LoadError::Variable {
        name: LOGGER_FORMAT,
        reason: "must be plain or json",
    })?;

    let logger_level = get(LOGGER_LEVEL, Some("INFO"))?;
    let logger_level = Level::from_str(&logger_level).map_err(|_| LoadError::Variable {
        name: LOGGER_LEVEL,
        reason: "must be NONE, ERROR, WARN, INFO, or DEBUG",
    })?;

    let postgres_port = get(POSTGRES_PORT, Some("5432"))?
        .parse::<u16>()
        .ok()
        .filter(|port| *port > 0)
        .ok_or(LoadError::Variable {
            name: POSTGRES_PORT,
            reason: "must be an integer from 1 to 65535",
        })?;

    let postgres_ssl_mode = get(POSTGRES_SSL_MODE, Some("prefer"))?;
    if !matches!(
        postgres_ssl_mode.as_str(),
        "disable" | "allow" | "prefer" | "require" | "verify-ca" | "verify-full"
    ) {
        return Err(LoadError::Variable {
            name: POSTGRES_SSL_MODE,
            reason: "must be disable, allow, prefer, require, verify-ca, or verify-full",
        });
    }

    let postgres_max_connections = positive(
        POSTGRES_MAX_CONNECTIONS,
        get(POSTGRES_MAX_CONNECTIONS, Some("10"))?,
        "must be a positive integer",
    )?;
    let postgres_max_connections =
        u32::try_from(postgres_max_connections).map_err(|_| LoadError::Variable {
            name: POSTGRES_MAX_CONNECTIONS,
            reason: "must fit in a 32-bit unsigned integer",
        })?;

    let cache_max_capacity = positive(
        CACHE_MAX_CAPACITY,
        get(CACHE_MAX_CAPACITY, Some("10000"))?,
        "must be a positive integer",
    )?;

    let postgres_connect_timeout = duration(
        POSTGRES_CONNECT_TIMEOUT,
        get(POSTGRES_CONNECT_TIMEOUT, Some("10000"))?,
    )?;
    let cache_individual_ttl = duration(
        CACHE_INDIVIDUAL_TTL,
        get(CACHE_INDIVIDUAL_TTL, Some("60000"))?,
    )?;
    let cache_list_ttl = duration(CACHE_LIST_TTL, get(CACHE_LIST_TTL, Some("10000"))?)?;
    let shutdown_timeout = duration(SHUTDOWN_TIMEOUT, get(SHUTDOWN_TIMEOUT, Some("15000"))?)?;

    let http_addr = address(HTTP_ADDR, get(HTTP_ADDR, Some("0.0.0.0:8080"))?)?;
    let grpc_addr = address(GRPC_ADDR, get(GRPC_ADDR, Some("0.0.0.0:8081"))?)?;

    Ok(AppEnv {
        logger_format,
        logger_level,
        postgres_host: nonblank(POSTGRES_HOST, get(POSTGRES_HOST, Some("127.0.0.1"))?)?,
        postgres_port,
        postgres_username: nonblank(POSTGRES_USERNAME, get(POSTGRES_USERNAME, None)?)?,
        // Preserve passwords exactly, including intentional whitespace or emptiness.
        postgres_password: get(POSTGRES_PASSWORD, None)?,
        postgres_database: nonblank(
            POSTGRES_DATABASE,
            get(POSTGRES_DATABASE, Some("master_database"))?,
        )?,
        postgres_ssl_mode,
        postgres_max_connections,
        postgres_connect_timeout,
        cache_max_capacity,
        cache_individual_ttl,
        cache_list_ttl,
        geoip_conf_path: nonblank(GEOIP_CONF_PATH, get(GEOIP_CONF_PATH, Some("GeoIP.conf"))?)?
            .into(),
        geoip_database_path: nonblank(
            GEOIP_DATABASE_PATH,
            get(GEOIP_DATABASE_PATH, Some("data/GeoLite2-City.mmdb"))?,
        )?
        .into(),
        http_addr,
        grpc_addr,
        shutdown_timeout,
    })
}

fn duration(name: &'static str, value: String) -> Result<Duration, LoadError> {
    let millis = value
        .parse::<u64>()
        .ok()
        .filter(|millis| *millis > 0)
        .ok_or(LoadError::Variable {
            name,
            reason: "must be a positive integer number of milliseconds",
        })?;
    Ok(Duration::from_millis(millis))
}

fn address(name: &'static str, value: String) -> Result<SocketAddr, LoadError> {
    value
        .parse::<SocketAddr>()
        .map_err(|_| LoadError::Variable {
            name,
            reason: "must be a socket address such as 0.0.0.0:8080",
        })
}

/// Configuration errors never carry rejected values or raw `.env` source lines.
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
mod tests {
    use super::*;

    fn base() -> HashMap<String, String> {
        HashMap::from([
            (POSTGRES_USERNAME.to_owned(), "user".to_owned()),
            (POSTGRES_PASSWORD.to_owned(), "secret".to_owned()),
        ])
    }

    fn parse(values: &HashMap<String, String>) -> Result<AppEnv, LoadError> {
        from_sources(values, |_| Ok(None))
    }

    fn expect_error(values: &HashMap<String, String>) -> LoadError {
        match parse(values) {
            Ok(_) => panic!("configuration should be rejected"),
            Err(error) => error,
        }
    }

    #[test]
    fn defaults_are_typed_and_bounded() {
        let env = parse(&base()).expect("default configuration");

        assert_eq!(env.logger_format, Format::Plain);
        assert_eq!(env.logger_level, Level::Info);
        assert_eq!(env.postgres_port, 5432);
        assert_eq!(env.postgres_ssl_mode, "prefer");
        assert_eq!(env.postgres_max_connections, 10);
        assert_eq!(env.postgres_connect_timeout, Duration::from_millis(10_000));
        assert_eq!(env.cache_max_capacity, 10_000);
        assert_eq!(env.cache_individual_ttl, Duration::from_millis(60_000));
        assert_eq!(env.cache_list_ttl, Duration::from_millis(10_000));
        assert_eq!(env.http_addr, "0.0.0.0:8080".parse().expect("http addr"));
        assert_eq!(env.grpc_addr, "0.0.0.0:8081".parse().expect("grpc addr"));
        assert_eq!(env.geoip_conf_path, std::path::Path::new("GeoIP.conf"));
        assert_eq!(
            env.geoip_database_path,
            std::path::Path::new("data/GeoLite2-City.mmdb")
        );
        assert_eq!(env.shutdown_timeout, Duration::from_millis(15_000));
    }

    #[test]
    fn process_values_override_file_values() {
        let mut values = base();
        values.insert(POSTGRES_HOST.to_owned(), "file-host".to_owned());
        values.insert(HTTP_ADDR.to_owned(), "127.0.0.1:9000".to_owned());

        let env = from_sources(&values, |name| match name {
            POSTGRES_HOST => Ok(Some("process-host".to_owned())),
            HTTP_ADDR => Ok(Some("127.0.0.1:9001".to_owned())),
            _ => Ok(None),
        })
        .expect("overridden configuration");

        assert_eq!(env.postgres_host, "process-host");
        assert_eq!(env.http_addr, "127.0.0.1:9001".parse().expect("http addr"));
    }

    #[test]
    fn invalid_values_report_only_names_and_safe_reasons() {
        let mut values = base();
        values.insert(POSTGRES_PORT.to_owned(), "0".to_owned());
        let error = expect_error(&values);
        assert!(matches!(
            error,
            LoadError::Variable {
                name: POSTGRES_PORT,
                ..
            }
        ));

        let mut values = base();
        values.insert(LOGGER_LEVEL.to_owned(), "LOUD".to_owned());
        let error = expect_error(&values);
        assert!(matches!(
            error,
            LoadError::Variable {
                name: LOGGER_LEVEL,
                ..
            }
        ));

        let mut values = base();
        values.insert(GRPC_ADDR.to_owned(), "not-an-address".to_owned());
        let error = expect_error(&values);
        assert!(matches!(
            error,
            LoadError::Variable {
                name: GRPC_ADDR,
                ..
            }
        ));

        let mut values = base();
        values.insert(SHUTDOWN_TIMEOUT.to_owned(), "0".to_owned());
        let error = expect_error(&values);
        assert!(matches!(
            error,
            LoadError::Variable {
                name: SHUTDOWN_TIMEOUT,
                ..
            }
        ));
    }

    #[test]
    fn geoip_paths_allow_overrides_and_reject_blank_values_without_echoing_them() {
        let mut values = base();
        values.insert(GEOIP_CONF_PATH.to_owned(), "/run/secrets/GeoIP.conf".into());
        values.insert(
            GEOIP_DATABASE_PATH.to_owned(),
            "/var/lib/geoip/city.mmdb".into(),
        );
        let configuration = parse(&values).unwrap();
        assert_eq!(
            configuration.geoip_conf_path,
            std::path::Path::new("/run/secrets/GeoIP.conf")
        );
        assert_eq!(
            configuration.geoip_database_path,
            std::path::Path::new("/var/lib/geoip/city.mmdb")
        );
        for name in [GEOIP_CONF_PATH, GEOIP_DATABASE_PATH] {
            values.insert(name.to_owned(), " ".into());
            assert!(
                matches!(expect_error(&values), LoadError::Variable { name: field, .. } if field == name)
            );
            values.insert(name.to_owned(), "valid-path".into());
        }
    }

    #[test]
    fn errors_do_not_render_secret_values() {
        let values = HashMap::from([(
            POSTGRES_PASSWORD.to_owned(),
            "super-secret-password".to_owned(),
        )]);
        let error = expect_error(&values);

        assert!(!error.to_string().contains("super-secret-password"));
        assert!(error.to_string().contains(POSTGRES_USERNAME));
    }
}
