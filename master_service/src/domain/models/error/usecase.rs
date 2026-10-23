use std::{error::Error as StdError, fmt};

use super::{GeoIpError, RepositoryError, ValidatorError};

/// Application failures preserve the classification of the capability that failed.
#[derive(Debug)]
pub enum UsecaseError {
    Validation(ValidatorError),
    Repository(RepositoryError),
    GeoIp(GeoIpError),
}

impl UsecaseError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::Validation(error) => error.code(),
            Self::Repository(error) => error.code(),
            Self::GeoIp(error) => error.code(),
        }
    }
}

impl fmt::Display for UsecaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(error) => error.fmt(formatter),
            Self::Repository(error) => error.fmt(formatter),
            Self::GeoIp(error) => error.fmt(formatter),
        }
    }
}

impl StdError for UsecaseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Validation(error) => Some(error),
            Self::Repository(error) => Some(error),
            Self::GeoIp(error) => Some(error),
        }
    }
}

impl From<ValidatorError> for UsecaseError {
    fn from(error: ValidatorError) -> Self {
        Self::Validation(error)
    }
}

impl From<RepositoryError> for UsecaseError {
    fn from(error: RepositoryError) -> Self {
        Self::Repository(error)
    }
}

impl From<GeoIpError> for UsecaseError {
    fn from(error: GeoIpError) -> Self {
        Self::GeoIp(error)
    }
}
