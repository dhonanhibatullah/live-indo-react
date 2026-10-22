use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use chrono::{DateTime, Utc};
use live_indo_proto::live_indo_react::master::v1::{
    self as proto, management_topic_service_server::ManagementTopicService,
};
use master_service::{
    domain::{
        models::{AppContext, RepositoryError, UsecaseError, entity::Topic},
        usecases::{
            UsecaseFuture,
            management::topic::{CreateTopic, ManageTopic, UpdateTopic},
        },
    },
    presentation::grpc::server::management::topic::TopicServer,
};
use prost::Message;
use tonic::{Code, Request};

struct FakeManageTopic {
    topic: Topic,
    updated: Mutex<Option<UpdateTopic>>,
    fail: AtomicBool,
}

impl FakeManageTopic {
    fn new() -> Self {
        Self {
            topic: topic(),
            updated: Mutex::new(None),
            fail: AtomicBool::new(false),
        }
    }

    fn fail(&self) {
        self.fail.store(true, Ordering::SeqCst);
    }
}

impl ManageTopic for FakeManageTopic {
    fn create_topic<'a>(&'a self, _: &'a AppContext, _: CreateTopic) -> UsecaseFuture<'a, Topic> {
        let topic = self.topic.clone();
        Box::pin(async move {
            if self.fail.load(Ordering::SeqCst) {
                Err(UsecaseError::Repository(RepositoryError::TopicNotFound))
            } else {
                Ok(topic)
            }
        })
    }

    fn update_topic<'a>(
        &'a self,
        _: &'a AppContext,
        input: UpdateTopic,
    ) -> UsecaseFuture<'a, Topic> {
        *self.updated.lock().expect("update lock") = Some(input);
        let topic = self.topic.clone();
        Box::pin(async move { Ok(topic) })
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
        selection_options: vec!["Yes".into(), "No".into()],
        is_active: true,
        created_at: DateTime::<Utc>::from_timestamp(0, 0).expect("timestamp"),
        updated_at: None,
        deleted_at: None,
    }
}

fn request<T>(message: T) -> Request<T> {
    let mut request = Request::new(message);
    request.extensions_mut().insert(AppContext::default());
    request
}

#[tokio::test]
async fn topic_server_maps_responses_and_optional_empty_selection_options() {
    let usecase = Arc::new(FakeManageTopic::new());
    let server = TopicServer::new(usecase.clone());

    let created = server
        .create_topic(request(proto::CreateTopicRequest {
            slug: "created".into(),
            name: "Created".into(),
            description: String::new(),
            polling_method_id: 1,
            selection_options: Vec::new(),
            is_active: true,
        }))
        .await
        .expect("create topic")
        .into_inner();
    assert_eq!(created.topic.expect("topic").slug, "topic");

    server
        .update_topic(request(proto::UpdateTopicRequest {
            id: 1,
            name: None,
            description: None,
            polling_method_id: None,
            selection_options: Some(proto::StringList { values: Vec::new() }),
            is_active: None,
        }))
        .await
        .expect("update topic");

    let updated = usecase.updated.lock().expect("update lock");
    assert_eq!(
        updated
            .as_ref()
            .and_then(|input| input.selection_options.clone()),
        Some(Vec::new())
    );
}

#[tokio::test]
async fn topic_server_maps_application_errors_to_typed_status() {
    let usecase = Arc::new(FakeManageTopic::new());
    usecase.fail();
    let server = TopicServer::new(usecase);

    let error = server
        .create_topic(request(proto::CreateTopicRequest {
            slug: "created".into(),
            name: "Created".into(),
            description: String::new(),
            polling_method_id: 1,
            selection_options: Vec::new(),
            is_active: true,
        }))
        .await
        .expect_err("missing topic");

    assert_eq!(error.code(), Code::NotFound);
    let detail = proto::Error::decode(error.details()).expect("typed detail");
    assert_eq!(detail.code, proto::ErrorCode::TopicNotFound as i32);
}
