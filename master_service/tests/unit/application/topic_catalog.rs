use std::sync::Arc;

use master_service::{
    application::topic::{PollingMethodQueries, TopicQueries},
    domain::{
        models::AppContext,
        usecases::{
            Page,
            topic::{ListPollingMethods, ListTopics, PollingMethodCatalog, TopicCatalog},
        },
    },
};

use super::support::{
    CapturingLogger, FakePollingMethodRepository, FakeTopicRepository, polling_method, topic,
};

#[tokio::test]
async fn topic_catalog_lists_only_active_topics_with_requested_filters() {
    let topics = Arc::new(FakeTopicRepository::new(topic()));
    let service = TopicQueries::new(topics.clone(), Arc::new(CapturingLogger::default()));

    let result = service
        .list_active_topics(
            &AppContext::default(),
            ListTopics {
                polling_method_id: Some(1),
                name_contains: Some("top".into()),
                page: Page {
                    limit: 10,
                    offset: 2,
                },
            },
        )
        .await
        .expect("list topics");
    assert_eq!(result.len(), 1);

    let args = topics.last_list.lock().expect("list lock");
    let args = args.as_ref().expect("list arguments");
    assert_eq!(args.is_active, Some(true));
    assert!(!args.include_deleted);
    assert_eq!(args.polling_method_id, Some(1));
    assert_eq!(args.name_contains.as_deref(), Some("top"));
    assert_eq!(args.pagination.limit, 10);
    assert_eq!(args.pagination.offset, 2);
}

#[tokio::test]
async fn polling_method_catalog_lists_only_active_methods() {
    let methods = Arc::new(FakePollingMethodRepository::new(Some(polling_method())));
    let service = PollingMethodQueries::new(methods.clone(), Arc::new(CapturingLogger::default()));

    let result = service
        .list_available_polling_methods(
            &AppContext::default(),
            ListPollingMethods {
                page: Page {
                    limit: 5,
                    offset: 0,
                },
            },
        )
        .await
        .expect("list methods");
    assert_eq!(result.len(), 1);

    let args = methods.last_list.lock().expect("list lock");
    let args = args.as_ref().expect("list arguments");
    assert_eq!(args.is_active, Some(true));
    assert!(!args.include_deleted);
    assert_eq!(args.pagination.limit, 5);
}

#[tokio::test]
async fn topic_catalog_returns_topics_by_slug() {
    let topics = Arc::new(FakeTopicRepository::new(topic()));
    let service = TopicQueries::new(topics, Arc::new(CapturingLogger::default()));

    assert_eq!(
        service
            .get_topic_by_slug(&AppContext::default(), "topic")
            .await
            .expect("get topic")
            .slug,
        "topic"
    );
}
