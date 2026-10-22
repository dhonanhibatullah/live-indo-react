use std::sync::Arc;

use axum::{
    extract::{FromRequestParts, Request},
    http::{HeaderMap, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::domain::{models::AppContext, usecases::misc::app_context::CreateAppContext};

use crate::presentation::http::dto::response::error::ApiError;

#[derive(Clone)]
pub struct RequestContext(pub AppContext);

pub(crate) async fn handle(
    context: Arc<dyn CreateAppContext>,
    mut request: Request,
    next: Next,
) -> Response {
    match build_context(context.as_ref(), request.headers()) {
        Ok(app_context) => {
            request.extensions_mut().insert(app_context);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}

fn build_context(
    context: &dyn CreateAppContext,
    headers: &HeaderMap,
) -> Result<AppContext, ApiError> {
    let trace = required_header(headers, "x-trace-id")?;
    let trace_id =
        Uuid::parse_str(&trace).map_err(|_| ApiError::bad_args("x-trace-id must be a UUID"))?;
    if trace_id.is_nil() {
        return Err(ApiError::bad_args("x-trace-id must be a non-zero UUID"));
    }
    let actor = optional_header(headers, "x-actor")?;
    Ok(context.create_app_context(actor, Some(trace_id)))
}

fn required_header(headers: &HeaderMap, key: &str) -> Result<String, ApiError> {
    single_header(headers, key)?.ok_or_else(|| ApiError::bad_args(format!("{key} is required")))
}

fn optional_header(headers: &HeaderMap, key: &str) -> Result<Option<String>, ApiError> {
    single_header(headers, key)
}

fn single_header(headers: &HeaderMap, key: &str) -> Result<Option<String>, ApiError> {
    let mut values = headers.get_all(key).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(ApiError::bad_args(format!("{key} must occur once")));
    }
    let value = value
        .to_str()
        .map_err(|_| ApiError::bad_args(format!("{key} must be ASCII text")))?;
    if value.trim().is_empty() {
        return Err(ApiError::bad_args(format!("{key} must not be empty")));
    }
    Ok(Some(value.to_owned()))
}

impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AppContext>()
            .cloned()
            .map(Self)
            .ok_or_else(|| ApiError::internal("request context is missing"))
    }
}
