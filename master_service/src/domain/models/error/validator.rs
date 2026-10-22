use std::{error::Error as StdError, fmt};

/// Validation failures with stable diagnostic codes.
///
/// Variants describe rules owned by the master-data domain. They carry no
/// transport status, raw input, or driver details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidatorError {
    BadArgs,

    EnvironmentRequired {
        key: &'static str,
    },
    EnvironmentInvalid {
        key: &'static str,
        reason: &'static str,
    },

    TopicSlugInvalid,
    TopicNameInvalid,
    TopicSelectionOptionsInvalid,
    TopicSelectionOptionsTooFew,
    TopicSelectionOptionsNotAllowed,

    PollingMethodCodeInvalid,
    PollingMethodNameInvalid,

    ProvinceKemendagriCodeInvalid,
    ProvinceNameInvalid,
    ProvinceBoundarySourceInvalid,

    CityKemendagriCodeInvalid,
    CityNameInvalid,
    CityBoundarySourceInvalid,
    CityProvinceCodeMismatch,
}

impl ValidatorError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::BadArgs => "BAD_ARGS",
            Self::EnvironmentRequired { .. } => "ENVIRONMENT_REQUIRED",
            Self::EnvironmentInvalid { .. } => "ENVIRONMENT_INVALID",
            Self::TopicSlugInvalid => "TOPIC_SLUG_INVALID",
            Self::TopicNameInvalid => "TOPIC_NAME_INVALID",
            Self::TopicSelectionOptionsInvalid => "TOPIC_SELECTION_OPTIONS_INVALID",
            Self::TopicSelectionOptionsTooFew => "TOPIC_SELECTION_OPTIONS_TOO_FEW",
            Self::TopicSelectionOptionsNotAllowed => "TOPIC_SELECTION_OPTIONS_NOT_ALLOWED",
            Self::PollingMethodCodeInvalid => "POLLING_METHOD_CODE_INVALID",
            Self::PollingMethodNameInvalid => "POLLING_METHOD_NAME_INVALID",
            Self::ProvinceKemendagriCodeInvalid => "PROVINCE_KEMENDAGRI_CODE_INVALID",
            Self::ProvinceNameInvalid => "PROVINCE_NAME_INVALID",
            Self::ProvinceBoundarySourceInvalid => "PROVINCE_BOUNDARY_SOURCE_INVALID",
            Self::CityKemendagriCodeInvalid => "CITY_KEMENDAGRI_CODE_INVALID",
            Self::CityNameInvalid => "CITY_NAME_INVALID",
            Self::CityBoundarySourceInvalid => "CITY_BOUNDARY_SOURCE_INVALID",
            Self::CityProvinceCodeMismatch => "CITY_PROVINCE_CODE_MISMATCH",
        }
    }
}

impl fmt::Display for ValidatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadArgs => formatter.write_str("invalid arguments"),
            Self::EnvironmentRequired { key } => write!(formatter, "{key} is required"),
            Self::EnvironmentInvalid { key, reason } => write!(formatter, "{key} {reason}"),
            Self::TopicSlugInvalid => formatter.write_str("topic slug is invalid"),
            Self::TopicNameInvalid => formatter.write_str("topic name is invalid"),
            Self::TopicSelectionOptionsInvalid => {
                formatter.write_str("topic selection options are invalid")
            }
            Self::TopicSelectionOptionsTooFew => formatter
                .write_str("topic selection options must contain at least two distinct labels"),
            Self::TopicSelectionOptionsNotAllowed => {
                formatter.write_str("topic selection options are not allowed for this method")
            }
            Self::PollingMethodCodeInvalid => formatter.write_str("polling method code is invalid"),
            Self::PollingMethodNameInvalid => formatter.write_str("polling method name is invalid"),
            Self::ProvinceKemendagriCodeInvalid => {
                formatter.write_str("province Kemendagri code is invalid")
            }
            Self::ProvinceNameInvalid => formatter.write_str("province name is invalid"),
            Self::ProvinceBoundarySourceInvalid => {
                formatter.write_str("province boundary source is invalid")
            }
            Self::CityKemendagriCodeInvalid => {
                formatter.write_str("city Kemendagri code is invalid")
            }
            Self::CityNameInvalid => formatter.write_str("city name is invalid"),
            Self::CityBoundarySourceInvalid => {
                formatter.write_str("city boundary source is invalid")
            }
            Self::CityProvinceCodeMismatch => {
                formatter.write_str("city Kemendagri code does not match its province")
            }
        }
    }
}

impl StdError for ValidatorError {}
