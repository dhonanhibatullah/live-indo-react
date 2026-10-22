use std::sync::Arc;

use master_service::{
    domain::{
        contracts::repository::{
            ProvinceRepository,
            province::{self as province_contract, ListArgs as ProvinceListArgs},
        },
        models::RepositoryError,
    },
    infrastructure::repository::PostgresProvince,
};

use crate::support::{
    database::Fixture,
    fixtures::{assert_boundaries_close, boundary, context, pagination, province_create},
    logger::CapturingLogger,
};

#[tokio::test]
async fn provinces_support_crud_boundaries_and_filters() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    let logger = Arc::new(CapturingLogger::default());
    let repository = PostgresProvince::new(fixture.database(), logger.clone());
    let context = context();

    let created = repository
        .create(&context, province_create("11", "Aceh"))
        .await
        .expect("create province");
    assert_eq!(created.kemendagri_code, "11");
    assert!(created.boundary_reference_date.is_some());
    assert_boundaries_close(&created.boundary, &boundary());

    assert_eq!(
        repository
            .find_by_id(&context, created.id)
            .await
            .expect("find province")
            .expect("province exists")
            .id,
        created.id
    );
    assert_eq!(
        repository
            .find_by_kemendagri_code(&context, "11")
            .await
            .expect("find province by code")
            .expect("province exists")
            .name,
        "Aceh"
    );
    assert_eq!(
        repository
            .find_province_by_coord(&context, 106.5, -6.5)
            .await
            .expect("find province by coordinate")
            .expect("coordinate is inside province")
            .id,
        created.id
    );
    assert_eq!(
        repository
            .find_province_by_coord(&context, 0.0, 0.0)
            .await
            .expect("find outside province"),
        None
    );
    assert!(matches!(
        repository
            .find_province_by_coord(&context, 181.0, 0.0)
            .await,
        Err(RepositoryError::BadArgs)
    ));
    assert!(matches!(
        repository
            .create(&context, province_create("11", "Duplicate"))
            .await,
        Err(RepositoryError::ProvinceKemendagriCodeConflict)
    ));

    repository
        .create(&context, province_create("12", "Sumatra Utara"))
        .await
        .expect("create second province");
    let listed = repository
        .list(
            &context,
            ProvinceListArgs {
                is_active: Some(true),
                name_contains: Some("aceh".into()),
                include_deleted: false,
                pagination: pagination(10, 0),
            },
        )
        .await
        .expect("filter provinces");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, created.id);

    let updated = repository
        .update(
            &context,
            created.id,
            province_contract::UpdateArgs {
                name: Some("Nanggroe Aceh Darussalam".into()),
                boundary: Some(boundary()),
                boundary_source: Some("updated source".into()),
                boundary_reference_date: Some(None),
                is_active: Some(false),
            },
        )
        .await
        .expect("update province");
    assert_eq!(updated.name, "Nanggroe Aceh Darussalam");
    assert_eq!(updated.boundary_source, "updated source");
    assert_eq!(updated.boundary_reference_date, None);
    assert!(!updated.is_active);
    assert!(updated.updated_at.is_some());

    let unchanged = repository
        .update(
            &context,
            created.id,
            province_contract::UpdateArgs {
                name: None,
                boundary: None,
                boundary_source: None,
                boundary_reference_date: None,
                is_active: None,
            },
        )
        .await
        .expect("no-op update province");
    assert_eq!(unchanged.updated_at, updated.updated_at);

    repository
        .delete(&context, created.id)
        .await
        .expect("delete province");
    assert!(
        repository
            .find_by_id(&context, created.id)
            .await
            .expect("find deleted province")
            .expect("deleted province remains readable")
            .deleted_at
            .is_some()
    );
    assert_eq!(
        repository
            .list(
                &context,
                ProvinceListArgs {
                    is_active: None,
                    name_contains: None,
                    include_deleted: false,
                    pagination: pagination(10, 0),
                },
            )
            .await
            .expect("list visible provinces")
            .len(),
        1
    );
    assert_eq!(
        repository
            .list(
                &context,
                ProvinceListArgs {
                    is_active: None,
                    name_contains: None,
                    include_deleted: true,
                    pagination: pagination(10, 0),
                },
            )
            .await
            .expect("list all provinces")
            .len(),
        2
    );
    assert!(matches!(
        repository
            .update(
                &context,
                created.id,
                province_contract::UpdateArgs {
                    name: Some("Hidden".into()),
                    boundary: None,
                    boundary_source: None,
                    boundary_reference_date: None,
                    is_active: None,
                },
            )
            .await,
        Err(RepositoryError::ProvinceNotFound)
    ));

    fixture.close().await;
}
