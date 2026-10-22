use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{
        Path, State,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    response::IntoResponse,
    routing::patch,
};

use crate::{
    domain::usecases::management::polling_method::UpdatePollingMethod,
    presentation::http::{
        Dependencies,
        dto::{
            request::{management::polling_method::UpdatePollingMethodRequest, path::IdPath},
            response::{error::ApiError, model},
        },
        middleware::RequestContext,
    },
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new().route(
        "/v1/polling-methods/{id}",
        patch(update_polling_method).delete(delete_polling_method),
    )
}

async fn update_polling_method(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<IdPath>, PathRejection>,
    payload: Result<Json<UpdatePollingMethodRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(path) = path?;
    let Json(input) = payload?;
    let mut input: UpdatePollingMethod = input.into();
    input.id = path.id;
    let method = dependencies
        .manage_polling_method
        .update_polling_method(&context, input)
        .await?;
    Ok(Json(model::polling_method(method)))
}

async fn delete_polling_method(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<IdPath>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    let Path(path) = path?;
    dependencies
        .manage_polling_method
        .delete_polling_method(&context, path.id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
