use bytes::Bytes;
use live_indo_proto::live_indo_react::master::v1::{self as proto, ErrorCode};
use prost::Message;
use tonic::{Code, Status};

use crate::domain::models::UsecaseError;

/// Map an application failure to a safe gRPC status with a typed detail.
pub(crate) fn usecase(error: UsecaseError) -> Status {
    let message = error.to_string();
    let (code, detail) = classify(error.code());
    status(code, detail, message)
}

pub(crate) fn bad_args(message: impl Into<String>) -> Status {
    status(Code::InvalidArgument, ErrorCode::BadArgs, message.into())
}

pub(crate) fn internal(message: impl Into<String>) -> Status {
    status(Code::Internal, ErrorCode::Failure, message.into())
}

fn status(code: Code, detail: ErrorCode, message: String) -> Status {
    let detail = proto::Error {
        code: detail as i32,
        message: message.clone(),
    };
    Status::with_details(code, message, Bytes::from(detail.encode_to_vec()))
}

fn classify(code: &str) -> (Code, ErrorCode) {
    match code {
        "BAD_ARGS" => (Code::InvalidArgument, ErrorCode::BadArgs),
        "NOT_FOUND" => (Code::NotFound, ErrorCode::NotFound),
        "CONFLICT" => (Code::AlreadyExists, ErrorCode::Conflict),
        "BAD_STATE" => (Code::FailedPrecondition, ErrorCode::BadState),
        "TIMEOUT" => (Code::DeadlineExceeded, ErrorCode::Timeout),
        "FAILURE" => (Code::Internal, ErrorCode::Failure),
        "UNIMPLEMENTED" => (Code::Unimplemented, ErrorCode::Unimplemented),
        "UNKNOWN" => (Code::Internal, ErrorCode::Unknown),

        "TOPIC_SLUG_INVALID" => (Code::InvalidArgument, ErrorCode::TopicSlugInvalid),
        "TOPIC_NAME_INVALID" => (Code::InvalidArgument, ErrorCode::TopicNameInvalid),
        "TOPIC_SELECTION_OPTIONS_INVALID" => (
            Code::InvalidArgument,
            ErrorCode::TopicSelectionOptionsInvalid,
        ),
        "TOPIC_SELECTION_OPTIONS_TOO_FEW" => (
            Code::InvalidArgument,
            ErrorCode::TopicSelectionOptionsTooFew,
        ),
        "TOPIC_SELECTION_OPTIONS_NOT_ALLOWED" => (
            Code::InvalidArgument,
            ErrorCode::TopicSelectionOptionsNotAllowed,
        ),
        "POLLING_METHOD_CODE_INVALID" => {
            (Code::InvalidArgument, ErrorCode::PollingMethodCodeInvalid)
        }
        "POLLING_METHOD_NAME_INVALID" => {
            (Code::InvalidArgument, ErrorCode::PollingMethodNameInvalid)
        }
        "PROVINCE_KEMENDAGRI_CODE_INVALID" => (
            Code::InvalidArgument,
            ErrorCode::ProvinceKemendagriCodeInvalid,
        ),
        "PROVINCE_NAME_INVALID" => (Code::InvalidArgument, ErrorCode::ProvinceNameInvalid),
        "PROVINCE_BOUNDARY_SOURCE_INVALID" => (
            Code::InvalidArgument,
            ErrorCode::ProvinceBoundarySourceInvalid,
        ),
        "CITY_KEMENDAGRI_CODE_INVALID" => {
            (Code::InvalidArgument, ErrorCode::CityKemendagriCodeInvalid)
        }
        "CITY_NAME_INVALID" => (Code::InvalidArgument, ErrorCode::CityNameInvalid),
        "CITY_BOUNDARY_SOURCE_INVALID" => {
            (Code::InvalidArgument, ErrorCode::CityBoundarySourceInvalid)
        }
        "CITY_PROVINCE_CODE_MISMATCH" => {
            (Code::InvalidArgument, ErrorCode::CityProvinceCodeMismatch)
        }

        "TOPIC_NOT_FOUND" => (Code::NotFound, ErrorCode::TopicNotFound),
        "TOPIC_SLUG_CONFLICT" => (Code::AlreadyExists, ErrorCode::TopicSlugConflict),
        "TOPIC_POLLING_METHOD_NOT_FOUND" => (Code::NotFound, ErrorCode::TopicPollingMethodNotFound),
        "POLLING_METHOD_NOT_FOUND" => (Code::NotFound, ErrorCode::PollingMethodNotFound),
        "POLLING_METHOD_CODE_CONFLICT" => {
            (Code::AlreadyExists, ErrorCode::PollingMethodCodeConflict)
        }
        "PROVINCE_NOT_FOUND" => (Code::NotFound, ErrorCode::ProvinceNotFound),
        "PROVINCE_KEMENDAGRI_CODE_CONFLICT" => (
            Code::AlreadyExists,
            ErrorCode::ProvinceKemendagriCodeConflict,
        ),
        "INDONESIA_BOUNDARY_NOT_FOUND" => (Code::NotFound, ErrorCode::IndonesiaBoundaryNotFound),
        "CITY_NOT_FOUND" => (Code::NotFound, ErrorCode::CityNotFound),
        "CITY_KEMENDAGRI_CODE_CONFLICT" => {
            (Code::AlreadyExists, ErrorCode::CityKemendagriCodeConflict)
        }
        "CITY_PROVINCE_NOT_FOUND" => (Code::NotFound, ErrorCode::CityProvinceNotFound),

        "GEOIP_CONFIGURATION_INVALID" => (Code::Internal, ErrorCode::GeoipConfigurationInvalid),
        "GEOIP_NOT_READY" => (Code::Unavailable, ErrorCode::GeoipNotReady),
        "GEOIP_DATABASE_INVALID" => (Code::Internal, ErrorCode::GeoipDatabaseInvalid),
        "GEOIP_DOWNLOAD_INVALID" => (Code::Internal, ErrorCode::GeoipDownloadInvalid),
        "GEOIP_UNAVAILABLE" => (Code::Unavailable, ErrorCode::GeoipUnavailable),
        "GEOIP_TIMEOUT" => (Code::DeadlineExceeded, ErrorCode::GeoipTimeout),
        "GEOIP_FAILURE" => (Code::Internal, ErrorCode::GeoipFailure),

        _ => (Code::Internal, ErrorCode::Unknown),
    }
}
