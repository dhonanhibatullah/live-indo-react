use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State, rejection::PathRejection},
    response::IntoResponse,
    routing::get,
};

use crate::presentation::http::{
    Dependencies,
    dto::{
        request::path::{CodePath, IdPath},
        response::{error::ApiError, model},
    },
    middleware::RequestContext,
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .route("/v1/provinces/{id}", get(get_province_by_id))
        .route(
            "/v1/provinces/by-code/{code}",
            get(get_province_by_kemendagri_code),
        )
}

async fn get_province_by_id(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<IdPath>, PathRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(path) = path?;
    let province = dependencies
        .get_province
        .get_province_by_id(&context, path.id)
        .await?;
    Ok(Json(model::province(province)))
}

async fn get_province_by_kemendagri_code(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<CodePath>, PathRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(path) = path?;
    let province = dependencies
        .get_province
        .get_province_by_kemendagri_code(&context, &path.code)
        .await?;
    Ok(Json(model::province(province)))
}
