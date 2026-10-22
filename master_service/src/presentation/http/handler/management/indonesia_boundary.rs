use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{State, rejection::JsonRejection},
    response::IntoResponse,
    routing::patch,
};

use crate::{
    domain::usecases::management::indonesia_boundary::UpdateIndonesiaBoundary,
    presentation::http::{
        Dependencies,
        dto::{
            request::management::indonesia_boundary::UpdateIndonesiaBoundaryRequest,
            response::{error::ApiError, model},
        },
        middleware::RequestContext,
    },
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new().route("/v1/indonesia-boundary", patch(update_indonesia_boundary))
}

async fn update_indonesia_boundary(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    payload: Result<Json<UpdateIndonesiaBoundaryRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(input) = payload?;
    let input = UpdateIndonesiaBoundary::try_from(input)?;
    let boundary = dependencies
        .manage_indonesia_boundary
        .update_indonesia_boundary(&context, input)
        .await?;
    Ok(Json(model::indonesia_boundary(boundary)))
}
