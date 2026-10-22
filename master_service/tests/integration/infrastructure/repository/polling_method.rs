use std::sync::Arc;

use master_service::{
    domain::{
        contracts::repository::{
            PollingMethodRepository,
            polling_method::{
                ListArgs as PollingMethodListArgs, UpdateArgs as PollingMethodUpdateArgs,
            },
        },
        models::RepositoryError,
    },
    infrastructure::repository::PostgresPollingMethod,
};

use crate::support::{
    database::Fixture,
    fixtures::{context, pagination, seed_polling_methods},
    logger::CapturingLogger,
};

#[tokio::test]
async fn polling_methods_support_reads_updates_and_soft_delete() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    seed_polling_methods(&fixture).await;
    let logger = Arc::new(CapturingLogger::default());
    let repository = PostgresPollingMethod::new(fixture.database(), logger.clone());
    let context = context();
    let id = fixture.polling_method_id("agree_disagree").await;

    let method = repository
        .find_by_id(&context, id)
        .await
        .expect("find polling method")
        .expect("polling method exists");
    assert_eq!(method.code, "agree_disagree");
    assert!(method.updated_at.is_none());
    assert!(method.deleted_at.is_none());

    assert_eq!(
        repository
            .find_by_code(&context, "agree_disagree")
            .await
            .expect("find by code")
            .expect("polling method exists")
            .id,
        id
    );
    assert_eq!(
        repository
            .find_by_id(&context, i64::MAX)
            .await
            .expect("missing polling method"),
        None
    );

    let all = repository
        .list(
            &context,
            PollingMethodListArgs {
                is_active: None,
                include_deleted: false,
                pagination: pagination(10, 0),
            },
        )
        .await
        .expect("list polling methods");
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].name, "Agree or disagree");
    assert_eq!(all[1].name, "Retired");
    assert_eq!(all[2].name, "Selection");

    let active = repository
        .list(
            &context,
            PollingMethodListArgs {
                is_active: Some(true),
                include_deleted: false,
                pagination: pagination(10, 0),
            },
        )
        .await
        .expect("list active polling methods");
    assert_eq!(active.len(), 2);

    let updated = repository
        .update(
            &context,
            id,
            PollingMethodUpdateArgs {
                name: Some("Agree / disagree".into()),
                description: Some("Updated".into()),
                is_active: Some(false),
            },
        )
        .await
        .expect("update polling method");
    assert_eq!(updated.name, "Agree / disagree");
    assert_eq!(updated.description, "Updated");
    assert!(!updated.is_active);
    assert!(updated.updated_at.is_some());

    let unchanged = repository
        .update(
            &context,
            id,
            PollingMethodUpdateArgs {
                name: None,
                description: None,
                is_active: None,
            },
        )
        .await
        .expect("no-op update polling method");
    assert_eq!(unchanged.updated_at, updated.updated_at);

    repository
        .delete(&context, id)
        .await
        .expect("delete polling method");
    let deleted = repository
        .find_by_id(&context, id)
        .await
        .expect("find deleted polling method")
        .expect("deleted polling method remains readable");
    assert!(deleted.deleted_at.is_some());
    assert_eq!(
        repository
            .find_by_code(&context, "agree_disagree")
            .await
            .expect("find deleted by code")
            .expect("deleted polling method remains readable by code")
            .id,
        id
    );
    assert_eq!(
        repository
            .list(
                &context,
                PollingMethodListArgs {
                    is_active: None,
                    include_deleted: false,
                    pagination: pagination(10, 0),
                },
            )
            .await
            .expect("list visible polling methods")
            .len(),
        2
    );
    assert_eq!(
        repository
            .list(
                &context,
                PollingMethodListArgs {
                    is_active: None,
                    include_deleted: true,
                    pagination: pagination(10, 0),
                },
            )
            .await
            .expect("list all polling methods")
            .len(),
        3
    );
    assert!(matches!(
        repository
            .update(
                &context,
                id,
                PollingMethodUpdateArgs {
                    name: Some("Hidden".into()),
                    description: None,
                    is_active: None,
                },
            )
            .await,
        Err(RepositoryError::PollingMethodNotFound)
    ));
    assert_eq!(
        logger
            .records()
            .last()
            .expect("polling method failure is logged")
            .tag,
        "repository/polling_method/update"
    );
    assert!(matches!(
        repository.delete(&context, id).await,
        Err(RepositoryError::PollingMethodNotFound)
    ));

    fixture.close().await;
}
