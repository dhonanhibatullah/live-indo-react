use std::sync::Arc;

use tonic::{
    Request, Status,
    metadata::MetadataMap,
    service::{Interceptor, InterceptorLayer},
};
use uuid::Uuid;

use crate::domain::usecases::misc::app_context::CreateAppContext;

use crate::presentation::grpc::mapper::error::bad_args;

const TRACE_ID: &str = "x-trace-id";
const ACTOR: &str = "x-actor";

#[derive(Clone)]
pub struct AppContextInterceptor {
    context: Arc<dyn CreateAppContext>,
}

impl AppContextInterceptor {
    pub fn new(context: Arc<dyn CreateAppContext>) -> Self {
        Self { context }
    }
}

impl Interceptor for AppContextInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let trace_id = required_metadata(request.metadata(), TRACE_ID)?;
        let trace_id =
            Uuid::parse_str(&trace_id).map_err(|_| bad_args("x-trace-id must be a UUID"))?;
        if trace_id.is_nil() {
            return Err(bad_args("x-trace-id must be a non-zero UUID"));
        }

        let actor = optional_metadata(request.metadata(), ACTOR)?;
        let context = self.context.create_app_context(actor, Some(trace_id));
        request.extensions_mut().insert(context);
        Ok(request)
    }
}

pub fn layer(context: Arc<dyn CreateAppContext>) -> InterceptorLayer<AppContextInterceptor> {
    InterceptorLayer::new(AppContextInterceptor::new(context))
}

fn required_metadata(metadata: &MetadataMap, key: &str) -> Result<String, Status> {
    single_metadata(metadata, key)?.ok_or_else(|| bad_args(format!("{key} is required")))
}

fn optional_metadata(metadata: &MetadataMap, key: &str) -> Result<Option<String>, Status> {
    single_metadata(metadata, key)
}

fn single_metadata(metadata: &MetadataMap, key: &str) -> Result<Option<String>, Status> {
    let mut values = metadata.get_all(key).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(bad_args(format!("{key} must occur once")));
    }
    let value = value
        .to_str()
        .map_err(|_| bad_args(format!("{key} must be ASCII text")))?;
    if value.trim().is_empty() {
        return Err(bad_args(format!("{key} must not be empty")));
    }
    Ok(Some(value.to_owned()))
}
