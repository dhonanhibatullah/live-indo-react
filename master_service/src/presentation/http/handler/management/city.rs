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
    domain::usecases::management::city::{CreateCity, UpdateCity},
    presentation::http::{
        Dependencies,
        dto::{
            request::{
                management::city::{CreateCityRequest, UpdateCityRequest},
                path::IdPath,
            },
            response::{error::ApiError, model},
        },
        middleware::RequestContext,
    },
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .route("/v1/cities", post(create_city))
        .route("/v1/cities/{id}", patch(update_city).delete(delete_city))
}

async fn create_city(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    payload: Result<Json<CreateCityRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(input) = payload?;
    let input = CreateCity::try_from(input)?;
    let city = dependencies
        .manage_city
        .create_city(&context, input)
        .await?;
    Ok((StatusCode::CREATED, Json(model::city(city))))
}

async fn update_city(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<IdPath>, PathRejection>,
    payload: Result<Json<UpdateCityRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(path) = path?;
    let Json(input) = payload?;
    let mut input = UpdateCity::try_from(input)?;
    input.id = path.id;
    let city = dependencies
        .manage_city
        .update_city(&context, input)
        .await?;
    Ok(Json(model::city(city)))
}

async fn delete_city(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<IdPath>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    let Path(path) = path?;
    dependencies
        .manage_city
        .delete_city(&context, path.id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
