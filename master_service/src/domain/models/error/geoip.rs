use std::{error::Error as StdError, fmt};

pub enum GeoIpError {
    InvalidConfiguration,
    NotReady,
    InvalidDatabase {
        source: Box<dyn StdError + Send + Sync>,
    },
    InvalidDownload {
        source: Box<dyn StdError + Send + Sync>,
    },
    Unavailable,
    Timeout {
        source: Box<dyn StdError + Send + Sync>,
    },
    Failure {
        source: Box<dyn StdError + Send + Sync>,
    },
}

impl GeoIpError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidConfiguration => "GEOIP_CONFIGURATION_INVALID",
            Self::NotReady => "GEOIP_NOT_READY",
            Self::InvalidDatabase { .. } => "GEOIP_DATABASE_INVALID",
            Self::InvalidDownload { .. } => "GEOIP_DOWNLOAD_INVALID",
            Self::Unavailable => "GEOIP_UNAVAILABLE",
            Self::Timeout { .. } => "GEOIP_TIMEOUT",
            Self::Failure { .. } => "GEOIP_FAILURE",
        }
    }
}

impl fmt::Display for GeoIpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidConfiguration => "GeoIP configuration is invalid",
            Self::NotReady => "GeoIP database is not ready",
            Self::InvalidDatabase { .. } => {
                "GeoIP database is invalid or has an unsupported edition"
            }
            Self::InvalidDownload { .. } => "GeoIP download is invalid or exceeds size limits",
            Self::Unavailable => "GeoIP download service is unavailable",
            Self::Timeout { .. } => "GeoIP operation timed out",
            Self::Failure { .. } => "GeoIP operation failed",
        })
    }
}

impl fmt::Debug for GeoIpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "[{}] {self}", self.code())
    }
}

impl StdError for GeoIpError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::InvalidDatabase { source }
            | Self::InvalidDownload { source }
            | Self::Timeout { source }
            | Self::Failure { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}
