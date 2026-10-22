use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1 as proto;
use master_service::{
    domain::{models::AppContext, usecases::misc::app_context::CreateAppContext},
    presentation::grpc::interceptor::AppContextInterceptor,
};
use prost::Message;
use tonic::{Code, Request, metadata::MetadataValue, service::Interceptor};
use uuid::Uuid;

struct FakeContext;

impl CreateAppContext for FakeContext {
    fn create_app_context(&self, actor: Option<String>, trace_id: Option<Uuid>) -> AppContext {
        AppContext {
            actor,
            trace_id,
            execution_id: Some(Uuid::from_u128(11)),
            transaction: None,
        }
    }
}

fn request(trace: Option<&str>, actor: Option<&str>) -> Request<()> {
    let mut request = Request::new(());
    if let Some(trace) = trace {
        request.metadata_mut().insert(
            "x-trace-id",
            MetadataValue::try_from(trace).expect("metadata"),
        );
    }
    if let Some(actor) = actor {
        request
            .metadata_mut()
            .insert("x-actor", MetadataValue::try_from(actor).expect("metadata"));
    }
    request
}

fn error_code(status: &tonic::Status) -> proto::ErrorCode {
    proto::Error::decode(status.details())
        .expect("typed error detail")
        .code
        .try_into()
        .expect("known error code")
}

#[test]
fn interceptor_builds_context_from_metadata() {
    let mut interceptor = AppContextInterceptor::new(Arc::new(FakeContext));
    let trace = Uuid::from_u128(22);
    let request = interceptor
        .call(request(Some(&trace.to_string()), Some("actor")))
        .expect("valid metadata");
    let context = request
        .extensions()
        .get::<AppContext>()
        .expect("context extension");

    assert_eq!(context.actor.as_deref(), Some("actor"));
    assert_eq!(context.trace_id, Some(trace));
    assert_eq!(context.execution_id, Some(Uuid::from_u128(11)));
}

#[test]
fn interceptor_rejects_missing_and_invalid_trace_metadata() {
    let mut interceptor = AppContextInterceptor::new(Arc::new(FakeContext));

    let missing = interceptor.call(request(None, None)).unwrap_err();
    assert_eq!(missing.code(), Code::InvalidArgument);
    assert_eq!(error_code(&missing), proto::ErrorCode::BadArgs);

    let invalid = interceptor
        .call(request(Some("not-a-uuid"), None))
        .unwrap_err();
    assert_eq!(invalid.code(), Code::InvalidArgument);

    let nil = interceptor
        .call(request(Some(&Uuid::nil().to_string()), None))
        .unwrap_err();
    assert_eq!(nil.code(), Code::InvalidArgument);
}

#[test]
fn interceptor_rejects_duplicate_trace_metadata() {
    let mut interceptor = AppContextInterceptor::new(Arc::new(FakeContext));
    let trace = Uuid::from_u128(33).to_string();
    let mut request = request(Some(&trace), None);
    request.metadata_mut().append(
        "x-trace-id",
        MetadataValue::try_from(trace).expect("metadata"),
    );

    let error = interceptor.call(request).unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
}
