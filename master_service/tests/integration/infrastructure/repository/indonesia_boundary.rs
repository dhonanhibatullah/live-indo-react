use std::sync::Arc;

use master_service::{
    domain::{
        contracts::repository::{IndonesiaBoundaryRepository, indonesia_boundary::UpdateArgs},
        models::RepositoryError,
    },
    infrastructure::repository::PostgresIndonesiaBoundary,
};
use pgdt::sqlx;

use crate::support::{database::Fixture, fixtures::context, logger::CapturingLogger};

#[tokio::test]
async fn boundary_supports_get_update_and_containment() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    let repository =
        PostgresIndonesiaBoundary::new(fixture.database(), Arc::new(CapturingLogger::default()));
    let context = context();

    assert!(repository.get(&context).await.unwrap().is_none());
    assert!(!repository.is_inside(&context, 110.0, 0.0).await.unwrap());
    assert!(matches!(
        repository
            .update(
                &context,
                UpdateArgs {
                    boundary: None,
                    boundary_source: Some("missing".into()),
                    boundary_reference_date: None,
                },
            )
            .await,
        Err(RepositoryError::IndonesiaBoundaryNotFound)
    ));

    sqlx::query(
        "INSERT INTO indonesia_boundary (boundary, boundary_source) VALUES (ST_GeomFromText('MULTIPOLYGON(((100 10, 120 10, 120 -10, 100 -10, 100 10)))', 4326), 'fixture')",
    )
    .execute(fixture.pool())
    .await
    .expect("insert national boundary");

    let boundary = repository
        .get(&context)
        .await
        .expect("get national boundary")
        .expect("national boundary exists");
    assert_eq!(boundary.id, 1);
    assert_eq!(boundary.boundary_source, "fixture");
    assert!(repository.is_inside(&context, 110.0, 0.0).await.unwrap());
    assert!(!repository.is_inside(&context, 0.0, 0.0).await.unwrap());
    assert!(matches!(
        repository.is_inside(&context, 181.0, 0.0).await,
        Err(RepositoryError::BadArgs)
    ));

    let updated = repository
        .update(
            &context,
            UpdateArgs {
                boundary: None,
                boundary_source: Some("updated source".into()),
                boundary_reference_date: Some(None),
            },
        )
        .await
        .expect("update national boundary");
    assert_eq!(updated.boundary_source, "updated source");
    assert_eq!(updated.boundary_reference_date, None);
    assert!(updated.updated_at.is_some());

    let unchanged = repository
        .update(
            &context,
            UpdateArgs {
                boundary: None,
                boundary_source: None,
                boundary_reference_date: None,
            },
        )
        .await
        .expect("no-op national boundary update");
    assert_eq!(unchanged.updated_at, updated.updated_at);

    fixture.close().await;
}
