use std::{error::Error as StdError, fmt};

/// Persistence failures with generic and master-data-specific classifications.
///
/// Operational variants retain their native cause; classification variants are
/// safe to map to stable diagnostic codes.
#[derive(Debug)]
pub enum RepositoryError {
    BadArgs,
    NotFound,
    Conflict,
    BadState,
    Timeout {
        source: Box<dyn StdError + Send + Sync + 'static>,
    },
    Failure {
        source: Box<dyn StdError + Send + Sync + 'static>,
    },
    Unimplemented,

    TopicNotFound,
    TopicSlugConflict,
    TopicPollingMethodNotFound,

    PollingMethodNotFound,
    PollingMethodCodeConflict,

    ProvinceNotFound,
    ProvinceKemendagriCodeConflict,

    IndonesiaBoundaryNotFound,

    CityNotFound,
    CityKemendagriCodeConflict,
    CityProvinceNotFound,
}

impl RepositoryError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::BadArgs => "BAD_ARGS",
            Self::NotFound => "NOT_FOUND",
            Self::Conflict => "CONFLICT",
            Self::BadState => "BAD_STATE",
            Self::Timeout { .. } => "TIMEOUT",
            Self::Failure { .. } => "FAILURE",
            Self::Unimplemented => "UNIMPLEMENTED",
            Self::TopicNotFound => "TOPIC_NOT_FOUND",
            Self::TopicSlugConflict => "TOPIC_SLUG_CONFLICT",
            Self::TopicPollingMethodNotFound => "TOPIC_POLLING_METHOD_NOT_FOUND",
            Self::PollingMethodNotFound => "POLLING_METHOD_NOT_FOUND",
            Self::PollingMethodCodeConflict => "POLLING_METHOD_CODE_CONFLICT",
            Self::ProvinceNotFound => "PROVINCE_NOT_FOUND",
            Self::ProvinceKemendagriCodeConflict => "PROVINCE_KEMENDAGRI_CODE_CONFLICT",
            Self::IndonesiaBoundaryNotFound => "INDONESIA_BOUNDARY_NOT_FOUND",
            Self::CityNotFound => "CITY_NOT_FOUND",
            Self::CityKemendagriCodeConflict => "CITY_KEMENDAGRI_CODE_CONFLICT",
            Self::CityProvinceNotFound => "CITY_PROVINCE_NOT_FOUND",
        }
    }
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BadArgs => "invalid arguments",
            Self::NotFound => "record was not found",
            Self::Conflict => "record conflicts with existing data",
            Self::BadState => "invalid state",
            Self::Timeout { .. } => "repository operation timed out",
            Self::Failure { .. } => "repository operation failed",
            Self::Unimplemented => "operation is not implemented",
            Self::TopicNotFound => "topic was not found",
            Self::TopicSlugConflict => "topic slug conflicts with existing data",
            Self::TopicPollingMethodNotFound => "topic polling method was not found",
            Self::PollingMethodNotFound => "polling method was not found",
            Self::PollingMethodCodeConflict => "polling method code conflicts with existing data",
            Self::ProvinceNotFound => "province was not found",
            Self::ProvinceKemendagriCodeConflict => {
                "province Kemendagri code conflicts with existing data"
            }
            Self::IndonesiaBoundaryNotFound => "Indonesia boundary was not found",
            Self::CityNotFound => "city was not found",
            Self::CityKemendagriCodeConflict => "city Kemendagri code conflicts with existing data",
            Self::CityProvinceNotFound => "city province was not found",
        })
    }
}

impl StdError for RepositoryError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Timeout { source } | Self::Failure { source } => {
                Some(source.as_ref() as &(dyn StdError + 'static))
            }
            _ => None,
        }
    }
}
