use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State, rejection::QueryRejection},
    response::IntoResponse,
    routing::get,
};

use crate::presentation::http::{
    Dependencies,
    dto::{
        request::coordinate::CoordinateQuery,
        response::{error::ApiError, geographic::InsideResponse},
    },
    middleware::RequestContext,
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new().route("/v1/indonesia/contains", get(is_inside_indonesia))
}

async fn is_inside_indonesia(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    query: Result<Query<CoordinateQuery>, QueryRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Query(query) = query?;
    let inside = dependencies
        .is_inside_indonesia
        .is_inside_indonesia(&context, query.longitude, query.latitude)
        .await?;
    Ok(Json(InsideResponse { inside }))
}
