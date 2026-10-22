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
    domain::usecases::topic::ListPollingMethods,
    presentation::http::{
        Dependencies,
        dto::{
            request::{path::CodePath, topic::polling_method::ListPollingMethodsQuery},
            response::{
                error::ApiError,
                model,
                topic::{PageResponse, PollingMethodListResponse},
            },
        },
        middleware::RequestContext,
    },
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .route("/v1/polling-methods", get(list_available_polling_methods))
        .route(
            "/v1/polling-methods/by-code/{code}",
            get(get_polling_method_by_code),
        )
}

async fn list_available_polling_methods(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    query: Result<Query<ListPollingMethodsQuery>, QueryRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Query(query) = query?;
    let page = query.page.into_page();
    let methods = dependencies
        .polling_method_catalog
        .list_available_polling_methods(&context, ListPollingMethods { page })
        .await?;
    Ok(Json(PollingMethodListResponse {
        data: methods.into_iter().map(model::polling_method).collect(),
        page: PageResponse::from(page),
    }))
}

async fn get_polling_method_by_code(
    State(dependencies): State<Arc<Dependencies>>,
    RequestContext(context): RequestContext,
    path: Result<Path<CodePath>, PathRejection>,
) -> Result<impl IntoResponse, ApiError> {
    let Path(path) = path?;
    let method = dependencies
        .polling_method_catalog
        .get_polling_method_by_code(&context, &path.code)
        .await?;
    Ok(Json(model::polling_method(method)))
}
