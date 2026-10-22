use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{
        Path, Query, State,
        rejection::{PathRejection, QueryRejection},
    },
    response::IntoResponse,
    routing::get,
};

use crate::{
    domain::usecases::topic::ListTopics,
    presentation::http::{
        Dependencies,
        dto::{
            request::{path::SlugPath, topic::catalog::ListTopicsQuery},
            response::{
                error::ApiError,
                model,
                topic::{PageResponse, TopicListResponse},
            },
        },
        middleware::RequestContext,
    },
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .route("/v1/topics", get(list_active_topics))
        .route("/v1/topics/by-slug/{slug}", get(get_topic_by_slug))
}

async fn list_active_topics(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    query: Result<Query<ListTopicsQuery>, QueryRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Query(query) = query?;
    let page = query.page.into_page();
    let topics = dependencies
        .topic_catalog
        .list_active_topics(
            &context,
            ListTopics {
                polling_method_id: query.polling_method_id,
                name_contains: query.name_contains,
                page,
            },
        )
        .await?;
    Ok(Json(TopicListResponse {
        data: topics.into_iter().map(model::topic).collect(),
        page: PageResponse::from(page),
    }))
}

async fn get_topic_by_slug(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<SlugPath>, PathRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(path) = path?;
    let topic = dependencies
        .topic_catalog
        .get_topic_by_slug(&context, &path.slug)
        .await?;
    Ok(Json(model::topic(topic)))
}
