use tonic::{Request, Status};

use crate::domain::models::AppContext;

use crate::presentation::grpc::mapper::error::internal;

pub(crate) fn app_context<T>(request: &Request<T>) -> Result<AppContext, Status> {
    request
        .extensions()
        .get::<AppContext>()
        .cloned()
        .ok_or_else(|| internal("request context is missing"))
}
