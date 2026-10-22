use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use live_indo_proto::live_indo_react::master::v1::{
    self as proto, management_topic_service_client::ManagementTopicServiceClient,
    management_topic_service_server::ManagementTopicServiceServer,
};
use master_service::{
    application::misc::AppContextService,
    domain::{
        models::{AppContext, entity::Topic},
        usecases::{
            UsecaseFuture,
            management::topic::{CreateTopic, ManageTopic, UpdateTopic},
        },
    },
    infrastructure::utility::UuidAppContextFactory,
    presentation::grpc::{interceptor, server::management::topic::TopicServer},
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{Request, metadata::MetadataValue, transport::Server};

struct FakeManageTopic {
    context: Arc<Mutex<Option<AppContext>>>,
}

impl FakeManageTopic {
    fn new(context: Arc<Mutex<Option<AppContext>>>) -> Self {
        Self { context }
    }
}

impl ManageTopic for FakeManageTopic {
    fn create_topic<'a>(
        &'a self,
        context: &'a AppContext,
        _: CreateTopic,
    ) -> UsecaseFuture<'a, Topic> {
        *self.context.lock().expect("context lock") = Some(context.clone());
        let topic = topic();
        Box::pin(async move { Ok(topic) })
    }

    fn update_topic<'a>(&'a self, _: &'a AppContext, _: UpdateTopic) -> UsecaseFuture<'a, Topic> {
        Box::pin(async {
            Err(master_service::domain::models::RepositoryError::Unimplemented.into())
        })
    }

    fn delete_topic<'a>(&'a self, _: &'a AppContext, _: i64) -> UsecaseFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

fn topic() -> Topic {
    Topic {
        id: 1,
        slug: "topic".into(),
        name: "Topic".into(),
        description: String::new(),
        polling_method_id: 1,
        selection_options: Vec::new(),
        is_active: true,
        created_at: DateTime::<Utc>::from_timestamp(0, 0).expect("timestamp"),
        updated_at: None,
        deleted_at: None,
    }
}

#[tokio::test]
async fn grpc_round_trip_carries_actor_and_trace_into_app_context() {
    let recorded = Arc::new(Mutex::new(None));
    let usecase = Arc::new(FakeManageTopic::new(Arc::clone(&recorded)));
    let context = Arc::new(AppContextService::new(Arc::new(
        UuidAppContextFactory::new(),
    )));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(async move {
        Server::builder()
            .layer(interceptor::layer(context))
            .add_service(ManagementTopicServiceServer::new(TopicServer::new(usecase)))
            .serve_with_incoming(TcpListenerStream::new(listener))
            .await
    });

    let mut client = ManagementTopicServiceClient::connect(format!("http://{address}"))
        .await
        .expect("connect test client");
    let mut request = Request::new(proto::CreateTopicRequest {
        slug: "created".into(),
        name: "Created".into(),
        description: String::new(),
        polling_method_id: 1,
        selection_options: Vec::new(),
        is_active: true,
    });
    request.metadata_mut().insert(
        "x-trace-id",
        MetadataValue::try_from("00000000-0000-0000-0000-00000000002a").expect("trace"),
    );
    request
        .metadata_mut()
        .insert("x-actor", MetadataValue::try_from("actor").expect("actor"));

    let response = client
        .create_topic(request)
        .await
        .expect("create topic response")
        .into_inner();
    assert_eq!(response.topic.expect("topic").slug, "topic");

    let context = recorded
        .lock()
        .expect("context lock")
        .clone()
        .expect("recorded context");
    assert_eq!(context.actor.as_deref(), Some("actor"));
    assert_eq!(context.trace_id, Some(uuid::Uuid::from_u128(42)));
    assert!(context.execution_id.is_some_and(|id| !id.is_nil()));
    server.abort();
}
