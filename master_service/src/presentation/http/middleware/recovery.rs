use axum::response::{IntoResponse, Response};
use tower_http::catch_panic::CatchPanicLayer;

use crate::presentation::http::dto::response::error::ApiError;

pub(crate) fn layer() -> CatchPanicLayer<fn(Box<dyn std::any::Any + Send>) -> Response> {
    CatchPanicLayer::custom(handle as fn(Box<dyn std::any::Any + Send>) -> Response)
}

fn handle(_: Box<dyn std::any::Any + Send>) -> Response {
    ApiError::internal("request failed").into_response()
}
