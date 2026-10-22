use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

use crate::domain::models::UsecaseError;

#[derive(Clone, Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    body: ErrorResponse,
}

impl ApiError {
    pub(crate) fn bad_args(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "BAD_ARGS", message)
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "FAILURE", message)
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "NOT_FOUND", message)
    }

    pub(crate) fn method_not_allowed(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::METHOD_NOT_ALLOWED,
            "METHOD_NOT_ALLOWED",
            message,
        )
    }

    fn new(status: StatusCode, code: &str, message: impl Into<String>) -> Self {
        Self {
            status,
            body: ErrorResponse {
                code: code.to_owned(),
                message: message.into(),
            },
        }
    }

    fn from_usecase(error: UsecaseError) -> Self {
        let (status, code) = classify(error.code());
        let message = match status {
            StatusCode::INTERNAL_SERVER_ERROR => "something went wrong".to_owned(),
            _ => error.to_string(),
        };
        Self {
            status,
            body: ErrorResponse {
                code: code.into(),
                message,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl From<UsecaseError> for ApiError {
    fn from(error: UsecaseError) -> Self {
        Self::from_usecase(error)
    }
}

impl From<JsonRejection> for ApiError {
    fn from(_: JsonRejection) -> Self {
        Self::bad_args("invalid request body")
    }
}

impl From<PathRejection> for ApiError {
    fn from(_: PathRejection) -> Self {
        Self::bad_args("invalid path parameter")
    }
}

impl From<QueryRejection> for ApiError {
    fn from(_: QueryRejection) -> Self {
        Self::bad_args("invalid query parameter")
    }
}

fn classify(code: &str) -> (StatusCode, &str) {
    match code {
        "BAD_ARGS"
        | "TOPIC_SLUG_INVALID"
        | "TOPIC_NAME_INVALID"
        | "TOPIC_SELECTION_OPTIONS_INVALID"
        | "TOPIC_SELECTION_OPTIONS_TOO_FEW"
        | "TOPIC_SELECTION_OPTIONS_NOT_ALLOWED"
        | "POLLING_METHOD_CODE_INVALID"
        | "POLLING_METHOD_NAME_INVALID"
        | "PROVINCE_KEMENDAGRI_CODE_INVALID"
        | "PROVINCE_NAME_INVALID"
        | "PROVINCE_BOUNDARY_SOURCE_INVALID"
        | "CITY_KEMENDAGRI_CODE_INVALID"
        | "CITY_NAME_INVALID"
        | "CITY_BOUNDARY_SOURCE_INVALID"
        | "CITY_PROVINCE_CODE_MISMATCH" => (StatusCode::BAD_REQUEST, code),

        "NOT_FOUND"
        | "TOPIC_NOT_FOUND"
        | "TOPIC_POLLING_METHOD_NOT_FOUND"
        | "POLLING_METHOD_NOT_FOUND"
        | "PROVINCE_NOT_FOUND"
        | "INDONESIA_BOUNDARY_NOT_FOUND"
        | "CITY_NOT_FOUND"
        | "CITY_PROVINCE_NOT_FOUND" => (StatusCode::NOT_FOUND, code),

        "CONFLICT"
        | "TOPIC_SLUG_CONFLICT"
        | "POLLING_METHOD_CODE_CONFLICT"
        | "PROVINCE_KEMENDAGRI_CODE_CONFLICT"
        | "CITY_KEMENDAGRI_CODE_CONFLICT" => (StatusCode::CONFLICT, code),

        "BAD_STATE" => (StatusCode::PRECONDITION_FAILED, code),
        "TIMEOUT" => (StatusCode::GATEWAY_TIMEOUT, code),
        "UNIMPLEMENTED" => (StatusCode::NOT_IMPLEMENTED, code),
        "FAILURE" | "UNKNOWN" => (StatusCode::INTERNAL_SERVER_ERROR, "FAILURE"),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "UNKNOWN"),
    }
}
