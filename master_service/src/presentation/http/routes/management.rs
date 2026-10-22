use std::sync::Arc;

use axum::Router;

use crate::presentation::http::{
    Dependencies,
    handler::management::{city, indonesia_boundary, polling_method, province, topic},
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .merge(topic::routes())
        .merge(polling_method::routes())
        .merge(province::routes())
        .merge(city::routes())
        .merge(indonesia_boundary::routes())
}
