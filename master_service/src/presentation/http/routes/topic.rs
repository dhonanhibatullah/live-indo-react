use std::sync::Arc;

use axum::Router;

use crate::presentation::http::{
    Dependencies,
    handler::topic::{catalog, polling_method},
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .merge(polling_method::routes())
        .merge(catalog::routes())
}
