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
        response::{error::ApiError, geographic::RegionLookupResponse, model},
    },
    middleware::RequestContext,
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new().route("/v1/regions/by-coordinate", get(get_region_by_coord))
}

async fn get_region_by_coord(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    query: Result<Query<CoordinateQuery>, QueryRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Query(query) = query?;
    let region = dependencies
        .get_region
        .get_region_by_coord(&context, query.longitude, query.latitude)
        .await?;
    Ok(Json(RegionLookupResponse {
        region: region.map(model::region),
    }))
}
