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
        request::{coordinate::CoordinateQuery, ip::IpQuery},
        response::{
            error::ApiError,
            geographic::{RegionByIpResponse, RegionLookupResponse},
            model,
        },
    },
    middleware::RequestContext,
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .route("/v1/regions/by-coordinate", get(get_region_by_coord))
        .route("/v1/regions/by-ip", get(get_region_by_ip))
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

async fn get_region_by_ip(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    query: Result<Query<IpQuery>, QueryRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Query(query) = query?;
    let result = dependencies
        .get_region
        .get_region_by_ip(&context, query.ip)
        .await?;
    Ok(Json(RegionByIpResponse {
        geoip: result.geoip.map(model::geoip),
        region: result.region.map(model::region),
    }))
}
