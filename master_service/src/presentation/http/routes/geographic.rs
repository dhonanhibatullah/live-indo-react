use std::sync::Arc;

use axum::Router;

use crate::presentation::http::{
    Dependencies,
    handler::geographic::{city, indonesia, province, region},
};

pub(crate) fn routes() -> Router<Arc<Dependencies>> {
    Router::new()
        .merge(province::routes())
        .merge(city::routes())
        .merge(region::routes())
        .merge(indonesia::routes())
}
