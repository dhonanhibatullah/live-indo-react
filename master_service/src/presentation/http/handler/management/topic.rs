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
    domain::usecases::management::topic::{CreateTopic, UpdateTopic},
    presentation::http::{
        Dependencies,
        dto::{
            request::{
                management::topic::{CreateTopicRequest, UpdateTopicRequest},
                path::IdPath,
            },
            response::{error::ApiError, model},
        },
        middleware::RequestContext,
    },
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .route("/v1/topics", post(create_topic))
        .route("/v1/topics/{id}", patch(update_topic).delete(delete_topic))
}

async fn create_topic(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    payload: Result<Json<CreateTopicRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Json(input) = payload?;
    let input: CreateTopic = input.into();
    let topic = dependencies
        .manage_topic
        .create_topic(&context, input)
        .await?;
    Ok((StatusCode::CREATED, Json(model::topic(topic))))
}

async fn update_topic(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<IdPath>, PathRejection>,
    payload: Result<Json<UpdateTopicRequest>, JsonRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(path) = path?;
    let Json(input) = payload?;
    let mut input: UpdateTopic = input.into();
    input.id = path.id;
    let topic = dependencies
        .manage_topic
        .update_topic(&context, input)
        .await?;
    Ok(Json(model::topic(topic)))
}

async fn delete_topic(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<IdPath>, PathRejection>,
) -> Result<StatusCode, ApiError> {
    let Path(path) = path?;
    dependencies
        .manage_topic
        .delete_topic(&context, path.id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
