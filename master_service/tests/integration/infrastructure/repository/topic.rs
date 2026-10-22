use std::sync::Arc;

use master_service::{
    domain::{
        contracts::repository::{
            TopicRepository,
            topic::{self as topic_contract, ListArgs as TopicListArgs},
        },
        models::{
            RepositoryError,
            logger::{Level, MetaValue},
        },
    },
    infrastructure::repository::PostgresTopic,
};

use crate::support::{
    database::Fixture,
    fixtures::{context, pagination, seed_polling_methods, topic_create, topic_update},
    logger::CapturingLogger,
};

#[tokio::test]
async fn topics_support_crud_filters_and_conflicts() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    seed_polling_methods(&fixture).await;
    let logger = Arc::new(CapturingLogger::default());
    let repository = PostgresTopic::new(fixture.database(), logger.clone());
    let context = context();
    let selection = fixture.polling_method_id("selection").await;
    let agree_disagree = fixture.polling_method_id("agree_disagree").await;

    let created = repository
        .create(&context, topic_create("topic-one", "Alpha", selection))
        .await
        .expect("create topic");
    assert_eq!(created.slug, "topic-one");
    assert_eq!(created.selection_options, vec!["Setuju", "Tidak setuju"]);
    assert!(created.updated_at.is_none());

    repository
        .create(&context, topic_create("topic-two", "Beta", agree_disagree))
        .await
        .expect("create second topic");
    repository
        .create(
            &context,
            topic_create("topic-three", "100% literal", selection),
        )
        .await
        .expect("create third topic");

    assert_eq!(
        repository
            .find_by_slug(&context, "topic-one")
            .await
            .expect("find by slug")
            .expect("topic exists")
            .id,
        created.id
    );
    assert_eq!(
        repository
            .find_by_id(&context, i64::MAX)
            .await
            .expect("missing topic"),
        None
    );

    assert!(matches!(
        repository
            .create(&context, topic_create("topic-one", "Duplicate", selection))
            .await,
        Err(RepositoryError::TopicSlugConflict)
    ));

    let records = logger.records();
    let conflict = records.last().expect("conflict is logged");
    assert_eq!(conflict.level, Level::Error);
    assert_eq!(conflict.tag, "repository/topic/create");
    let meta = conflict.meta.as_ref().expect("conflict metadata");
    assert_eq!(
        meta.get("error_code"),
        Some(&MetaValue::String("TOPIC_SLUG_CONFLICT".into()))
    );
    assert_eq!(
        meta.get("constraint"),
        Some(&MetaValue::String("uq_topics_slug".into()))
    );
    assert_eq!(
        meta.get("sql_state"),
        Some(&MetaValue::String("23505".into()))
    );
    assert!(!conflict.message.contains("topic-one"));
    assert!(!format!("{conflict:?}").contains("topic-one"));

    assert!(matches!(
        repository
            .create(
                &context,
                topic_create("missing-method", "Missing", i64::MAX)
            )
            .await,
        Err(RepositoryError::TopicPollingMethodNotFound)
    ));

    let filtered = repository
        .list(
            &context,
            TopicListArgs {
                polling_method_id: Some(selection),
                is_active: Some(true),
                name_contains: Some("alpha".into()),
                include_deleted: false,
                pagination: pagination(10, 0),
            },
        )
        .await
        .expect("filter topics");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, created.id);

    let literal = repository
        .list(
            &context,
            TopicListArgs {
                polling_method_id: None,
                is_active: None,
                name_contains: Some("100%".into()),
                include_deleted: false,
                pagination: pagination(10, 0),
            },
        )
        .await
        .expect("literal percent search");
    assert_eq!(literal.len(), 1);
    assert_eq!(literal[0].slug, "topic-three");

    let page = repository
        .list(
            &context,
            TopicListArgs {
                polling_method_id: None,
                is_active: None,
                name_contains: None,
                include_deleted: false,
                pagination: pagination(1, 1),
            },
        )
        .await
        .expect("paginate topics");
    assert_eq!(page.len(), 1);
    assert_eq!(page[0].slug, "topic-one");

    let updated = repository
        .update(&context, created.id, topic_update())
        .await
        .expect("update topic");
    assert_eq!(updated.name, "Updated topic");
    assert_eq!(updated.selection_options, vec!["Yes", "No"]);
    assert!(updated.updated_at.is_some());

    let unchanged = repository
        .update(
            &context,
            created.id,
            topic_contract::UpdateArgs {
                name: None,
                description: None,
                polling_method_id: None,
                selection_options: None,
                is_active: None,
            },
        )
        .await
        .expect("no-op update topic");
    assert_eq!(unchanged.updated_at, updated.updated_at);

    repository
        .delete(&context, created.id)
        .await
        .expect("delete topic");
    assert!(
        repository
            .find_by_id(&context, created.id)
            .await
            .expect("find deleted topic")
            .expect("deleted topic remains readable")
            .deleted_at
            .is_some()
    );
    assert!(
        repository
            .find_by_slug(&context, "topic-one")
            .await
            .expect("find deleted by slug")
            .is_some()
    );
    assert_eq!(
        repository
            .list(
                &context,
                TopicListArgs {
                    polling_method_id: None,
                    is_active: None,
                    name_contains: None,
                    include_deleted: false,
                    pagination: pagination(10, 0),
                },
            )
            .await
            .expect("list visible topics")
            .len(),
        2
    );
    assert_eq!(
        repository
            .list(
                &context,
                TopicListArgs {
                    polling_method_id: None,
                    is_active: None,
                    name_contains: None,
                    include_deleted: true,
                    pagination: pagination(10, 0),
                },
            )
            .await
            .expect("list all topics")
            .len(),
        3
    );
    assert!(matches!(
        repository
            .create(&context, topic_create("topic-one", "Reserved", selection))
            .await,
        Err(RepositoryError::TopicSlugConflict)
    ));
    assert!(matches!(
        repository
            .update(&context, created.id, topic_update())
            .await,
        Err(RepositoryError::TopicNotFound)
    ));
    assert!(matches!(
        repository.delete(&context, created.id).await,
        Err(RepositoryError::TopicNotFound)
    ));

    fixture.close().await;
}
