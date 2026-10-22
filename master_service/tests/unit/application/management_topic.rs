use std::sync::Arc;

use master_service::{
    application::management::TopicManagement,
    domain::{
        models::{
            AppContext, UsecaseError,
            logger::{Level, MetaValue},
        },
        usecases::management::topic::{CreateTopic, ManageTopic},
    },
};

use super::support::{
    CapturingLogger, FakePollingMethodRepository, FakeTopicRepository, TestValidator,
    polling_method, topic,
};

fn create_topic() -> CreateTopic {
    CreateTopic {
        slug: "created".into(),
        name: "Created".into(),
        description: "Description".into(),
        polling_method_id: 1,
        selection_options: vec!["Yes".into(), "No".into()],
        is_active: true,
    }
}

#[tokio::test]
async fn topic_management_validates_and_persists_explicit_requests() {
    let topics = Arc::new(FakeTopicRepository::new(topic()));
    let methods = Arc::new(FakePollingMethodRepository::new(Some(polling_method())));
    let validator = Arc::new(TestValidator::new());
    let logger = Arc::new(CapturingLogger::default());
    let service = TopicManagement::new(topics.clone(), methods, validator, logger.clone());

    service
        .create_topic(&AppContext::default(), create_topic())
        .await
        .expect("create topic");

    let created = topics.created.lock().expect("created lock");
    let created = created.as_ref().expect("create arguments");
    assert_eq!(created.slug, "created");
    assert_eq!(created.polling_method_id, 1);
    assert_eq!(created.selection_options, ["Yes", "No"]);

    let records = logger.records();
    let record = records.last().expect("success log");
    assert_eq!(record.level, Level::Info);
    assert_eq!(record.tag, "management/topic/create");
    assert_eq!(record.message, "topic created");
    assert_eq!(
        record.meta.as_ref().and_then(|meta| meta.get("topic_id")),
        Some(&MetaValue::Integer(1))
    );
}

#[tokio::test]
async fn topic_management_stops_invalid_input_before_persistence() {
    let topics = Arc::new(FakeTopicRepository::new(topic()));
    let methods = Arc::new(FakePollingMethodRepository::new(Some(polling_method())));
    let validator = Arc::new(TestValidator::new());
    validator.reject_slug();
    let service = TopicManagement::new(
        topics.clone(),
        methods,
        validator,
        Arc::new(CapturingLogger::default()),
    );

    let error = service
        .create_topic(&AppContext::default(), create_topic())
        .await
        .expect_err("invalid slug must fail");
    assert!(matches!(error, UsecaseError::Validation(_)));
    assert_eq!(error.code(), "TOPIC_SLUG_INVALID");
    assert!(topics.created.lock().expect("created lock").is_none());
}

#[tokio::test]
async fn topic_management_rejects_a_missing_polling_method() {
    let topics = Arc::new(FakeTopicRepository::new(topic()));
    let methods = Arc::new(FakePollingMethodRepository::new(Some(polling_method())));
    methods.clear();
    let logger = Arc::new(CapturingLogger::default());
    let service = TopicManagement::new(
        topics,
        methods,
        Arc::new(TestValidator::new()),
        logger.clone(),
    );

    let error = service
        .create_topic(&AppContext::default(), create_topic())
        .await
        .expect_err("missing method must fail");
    assert_eq!(error.code(), "TOPIC_POLLING_METHOD_NOT_FOUND");

    let records = logger.records();
    let record = records.last().expect("application error log");
    assert_eq!(record.level, Level::Error);
    assert_eq!(record.tag, "management/topic/create");
    assert_eq!(record.message, "topic polling method was not found");
}
