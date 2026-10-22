use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{
        Path, State,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{patch, post},
};

use crate::{
    domain::usecases::management::province::{CreateProvince, UpdateProvince},
    presentation::http::{
        Dependencies,
        dto::{
            request::{
                management::province::{CreateProvinceRequest, UpdateProvinceRequest},
                path::IdPath,
            },
            response::{error::ApiError, model},
        },
        middleware::RequestContext,
    },
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .route("/v1/provinces", post(create_province))
        .route(
            "/v1/provinces/{id}",
            patch(update_province).delete(delete_province),
        )
}

async fn create_province(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    payload: Result<Json<CreateProvinceRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(input) = payload?;
    let input = CreateProvince::try_from(input)?;
    let province = dependencies
        .manage_province
        .create_province(&context, input)
        .await?;
    Ok((StatusCode::CREATED, Json(model::province(province))))
}

async fn update_province(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<IdPath>, PathRejection>,
    payload: Result<Json<UpdateProvinceRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(path) = path?;
    let Json(input) = payload?;
    let mut input = UpdateProvince::try_from(input)?;
    input.id = path.id;
    let province = dependencies
        .manage_province
        .update_province(&context, input)
        .await?;
    Ok(Json(model::province(province)))
}

async fn delete_province(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<IdPath>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    let Path(path) = path?;
    dependencies
        .manage_province
        .delete_province(&context, path.id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
