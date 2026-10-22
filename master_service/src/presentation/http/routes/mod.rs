mod geographic;
mod management;
mod topic;

use std::sync::Arc;

use axum::{Router, extract::DefaultBodyLimit, middleware};

use super::{
    Dependencies,
    dto::response::error::ApiError,
    middleware::{handle, layer},
};

pub fn router(dependencies: Dependencies) -> Router {
    let context = Arc::clone(&dependencies.create_app_context);
    let state = Arc::new(dependencies);

    Router::new()
        .merge(management::routes())
        .merge(geographic::routes())
        .merge(topic::routes())
        .fallback(not_found)
        .method_not_allowed_fallback(method_not_allowed)
        .with_state(state)
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn(move |request, next| {
            let context = Arc::clone(&context);
            async move { handle(context, request, next).await }
        }))
        .layer(layer())
}

async fn not_found() -> ApiError {
    ApiError::not_found("the requested route does not exist")
}

async fn method_not_allowed() -> ApiError {
    ApiError::method_not_allowed("the HTTP method is not supported for this route")
}
