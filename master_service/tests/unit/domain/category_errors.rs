use std::{error::Error as StdError, io};

use master_service::domain::models::error::{RepositoryError, UsecaseError, ValidatorError};

#[test]
fn validator_codes_are_stable() {
    for (error, code) in [
        (ValidatorError::BadArgs, "BAD_ARGS"),
        (
            ValidatorError::EnvironmentRequired { key: "PORT" },
            "ENVIRONMENT_REQUIRED",
        ),
        (
            ValidatorError::EnvironmentInvalid {
                key: "PORT",
                reason: "must be a number",
            },
            "ENVIRONMENT_INVALID",
        ),
        (ValidatorError::TopicSlugInvalid, "TOPIC_SLUG_INVALID"),
        (ValidatorError::TopicNameInvalid, "TOPIC_NAME_INVALID"),
        (
            ValidatorError::TopicSelectionOptionsInvalid,
            "TOPIC_SELECTION_OPTIONS_INVALID",
        ),
        (
            ValidatorError::TopicSelectionOptionsTooFew,
            "TOPIC_SELECTION_OPTIONS_TOO_FEW",
        ),
        (
            ValidatorError::TopicSelectionOptionsNotAllowed,
            "TOPIC_SELECTION_OPTIONS_NOT_ALLOWED",
        ),
        (
            ValidatorError::PollingMethodCodeInvalid,
            "POLLING_METHOD_CODE_INVALID",
        ),
        (
            ValidatorError::PollingMethodNameInvalid,
            "POLLING_METHOD_NAME_INVALID",
        ),
        (
            ValidatorError::ProvinceKemendagriCodeInvalid,
            "PROVINCE_KEMENDAGRI_CODE_INVALID",
        ),
        (ValidatorError::ProvinceNameInvalid, "PROVINCE_NAME_INVALID"),
        (
            ValidatorError::ProvinceBoundarySourceInvalid,
            "PROVINCE_BOUNDARY_SOURCE_INVALID",
        ),
        (
            ValidatorError::CityKemendagriCodeInvalid,
            "CITY_KEMENDAGRI_CODE_INVALID",
        ),
        (ValidatorError::CityNameInvalid, "CITY_NAME_INVALID"),
        (
            ValidatorError::CityBoundarySourceInvalid,
            "CITY_BOUNDARY_SOURCE_INVALID",
        ),
        (
            ValidatorError::CityProvinceCodeMismatch,
            "CITY_PROVINCE_CODE_MISMATCH",
        ),
    ] {
        assert_eq!(error.code(), code);
    }
}

#[test]
fn validator_messages_are_safe_and_environment_keys_are_preserved() {
    assert_eq!(ValidatorError::BadArgs.to_string(), "invalid arguments");
    assert_eq!(
        ValidatorError::EnvironmentRequired {
            key: "DATABASE_URL"
        }
        .to_string(),
        "DATABASE_URL is required"
    );
    assert_eq!(
        ValidatorError::EnvironmentInvalid {
            key: "DATABASE_URL",
            reason: "must be a PostgreSQL URL",
        }
        .to_string(),
        "DATABASE_URL must be a PostgreSQL URL"
    );
    assert_eq!(
        ValidatorError::CityProvinceCodeMismatch.to_string(),
        "city Kemendagri code does not match its province"
    );
}

#[test]
fn repository_generic_codes_are_stable() {
    for (error, code) in [
        (RepositoryError::BadArgs, "BAD_ARGS"),
        (RepositoryError::NotFound, "NOT_FOUND"),
        (RepositoryError::Conflict, "CONFLICT"),
        (RepositoryError::BadState, "BAD_STATE"),
        (RepositoryError::Unimplemented, "UNIMPLEMENTED"),
    ] {
        assert_eq!(error.code(), code);
    }
}

#[test]
fn repository_specific_codes_are_stable() {
    for (error, code) in [
        (RepositoryError::TopicNotFound, "TOPIC_NOT_FOUND"),
        (RepositoryError::TopicSlugConflict, "TOPIC_SLUG_CONFLICT"),
        (
            RepositoryError::TopicPollingMethodNotFound,
            "TOPIC_POLLING_METHOD_NOT_FOUND",
        ),
        (
            RepositoryError::PollingMethodNotFound,
            "POLLING_METHOD_NOT_FOUND",
        ),
        (
            RepositoryError::PollingMethodCodeConflict,
            "POLLING_METHOD_CODE_CONFLICT",
        ),
        (RepositoryError::ProvinceNotFound, "PROVINCE_NOT_FOUND"),
        (
            RepositoryError::IndonesiaBoundaryNotFound,
            "INDONESIA_BOUNDARY_NOT_FOUND",
        ),
        (
            RepositoryError::ProvinceKemendagriCodeConflict,
            "PROVINCE_KEMENDAGRI_CODE_CONFLICT",
        ),
        (RepositoryError::CityNotFound, "CITY_NOT_FOUND"),
        (
            RepositoryError::CityKemendagriCodeConflict,
            "CITY_KEMENDAGRI_CODE_CONFLICT",
        ),
        (
            RepositoryError::CityProvinceNotFound,
            "CITY_PROVINCE_NOT_FOUND",
        ),
    ] {
        assert_eq!(error.code(), code);
    }
}

#[test]
fn usecase_errors_preserve_nested_classification_and_source() {
    let validation = UsecaseError::from(ValidatorError::TopicSlugInvalid);
    assert_eq!(validation.code(), "TOPIC_SLUG_INVALID");
    assert_eq!(validation.to_string(), "topic slug is invalid");
    assert!(validation.source().is_some());

    let repository = UsecaseError::from(RepositoryError::TopicNotFound);
    assert_eq!(repository.code(), "TOPIC_NOT_FOUND");
    assert_eq!(repository.to_string(), "topic was not found");
    assert!(repository.source().is_some());
}

#[test]
fn repository_operational_errors_preserve_native_sources() {
    let timeout = RepositoryError::Timeout {
        source: Box::new(io::Error::new(io::ErrorKind::TimedOut, "deadline reached")),
    };
    assert_eq!(timeout.code(), "TIMEOUT");
    assert_eq!(timeout.to_string(), "repository operation timed out");
    let source = timeout
        .source()
        .unwrap()
        .downcast_ref::<io::Error>()
        .unwrap();
    assert_eq!(source.kind(), io::ErrorKind::TimedOut);

    let failure = RepositoryError::Failure {
        source: Box::new(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "connection closed",
        )),
    };
    assert_eq!(failure.code(), "FAILURE");
    assert_eq!(failure.to_string(), "repository operation failed");
    let source = failure
        .source()
        .unwrap()
        .downcast_ref::<io::Error>()
        .unwrap();
    assert_eq!(source.kind(), io::ErrorKind::BrokenPipe);
}

#[test]
fn repository_classification_errors_have_no_source() {
    assert!(RepositoryError::TopicNotFound.source().is_none());
    assert!(
        RepositoryError::IndonesiaBoundaryNotFound
            .source()
            .is_none()
    );
    assert!(
        RepositoryError::CityKemendagriCodeConflict
            .source()
            .is_none()
    );
}
