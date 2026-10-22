use std::sync::Arc;

use master_service::{
    domain::{
        contracts::repository::TopicRepository,
        models::{RepositoryError, entity::Topic},
    },
    infrastructure::repository::PostgresTopic,
};
use pgdt::sqlx;

use crate::support::{
    database::Fixture,
    fixtures::{context, seed_polling_methods, topic_create},
    logger::CapturingLogger,
};

#[derive(Debug)]
enum TransactionError {
    Driver(sqlx::Error),
    Repository(RepositoryError),
    Rollback,
}

impl From<sqlx::Error> for TransactionError {
    fn from(source: sqlx::Error) -> Self {
        Self::Driver(source)
    }
}

impl From<RepositoryError> for TransactionError {
    fn from(source: RepositoryError) -> Self {
        Self::Repository(source)
    }
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Driver(source) => write!(formatter, "driver error: {source}"),
            Self::Repository(source) => write!(formatter, "repository error: {source}"),
            Self::Rollback => formatter.write_str("rollback requested"),
        }
    }
}

impl std::error::Error for TransactionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Driver(source) => Some(source),
            Self::Repository(source) => Some(source),
            Self::Rollback => None,
        }
    }
}

#[tokio::test]
async fn repositories_use_trait_objects_and_shared_transactions() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    seed_polling_methods(&fixture).await;
    let database = fixture.database();
    let logger = Arc::new(CapturingLogger::default());
    let repository: Arc<dyn TopicRepository> =
        Arc::new(PostgresTopic::new(database.clone(), logger));
    let context = context();
    let selection = fixture.polling_method_id("selection").await;

    let committed: Topic = database
        .with_tx(&context, |context| {
            let repository = Arc::clone(&repository);
            async move {
                repository
                    .create(&context, topic_create("committed", "Committed", selection))
                    .await
                    .map_err(TransactionError::from)
            }
        })
        .await
        .expect("commit repository transaction");
    assert!(
        repository
            .find_by_slug(&context, &committed.slug)
            .await
            .expect("find committed topic")
            .is_some()
    );

    let rollback: Result<Topic, TransactionError> = database
        .with_tx(&context, |context| {
            let repository = Arc::clone(&repository);
            async move {
                repository
                    .create(&context, topic_create("rolled-back", "Rollback", selection))
                    .await?;
                Err(TransactionError::Rollback)
            }
        })
        .await;
    assert!(matches!(rollback, Err(TransactionError::Rollback)));
    assert_eq!(
        repository
            .find_by_slug(&context, "rolled-back")
            .await
            .expect("find rolled-back topic"),
        None
    );

    fixture.close().await;
}

#[tokio::test]
async fn repository_errors_roll_back_transactions() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    seed_polling_methods(&fixture).await;
    let database = fixture.database();
    let repository: Arc<dyn TopicRepository> = Arc::new(PostgresTopic::new(
        database.clone(),
        Arc::new(CapturingLogger::default()),
    ));
    let context = context();
    let selection = fixture.polling_method_id("selection").await;

    repository
        .create(&context, topic_create("existing", "Existing", selection))
        .await
        .expect("create existing topic");

    let result: Result<Topic, TransactionError> = database
        .with_tx(&context, |context| {
            let repository = Arc::clone(&repository);
            async move {
                repository
                    .create(
                        &context,
                        topic_create("rolled-back-error", "Rollback error", selection),
                    )
                    .await?;
                let duplicate = repository
                    .create(&context, topic_create("existing", "Duplicate", selection))
                    .await?;
                Ok(duplicate)
            }
        })
        .await;

    assert!(matches!(
        result,
        Err(TransactionError::Repository(
            RepositoryError::TopicSlugConflict
        ))
    ));
    assert_eq!(
        repository
            .find_by_slug(&context, "rolled-back-error")
            .await
            .expect("find rolled-back topic"),
        None
    );

    fixture.close().await;
}
