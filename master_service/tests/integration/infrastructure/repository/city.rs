use std::sync::Arc;

use master_service::{
    domain::{
        contracts::repository::{
            CityRepository, ProvinceRepository,
            city::{self as city_contract, ListArgs as CityListArgs},
        },
        models::{RepositoryError, entity::CityKind},
    },
    infrastructure::repository::{PostgresCity, PostgresProvince},
};

use crate::support::{
    database::Fixture,
    fixtures::{
        assert_boundaries_close, boundary, city_create, context, pagination, province_create,
    },
    logger::CapturingLogger,
};

#[tokio::test]
async fn cities_support_crud_filters_and_parent_errors() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    let logger = Arc::new(CapturingLogger::default());
    let database = fixture.database();
    let provinces = PostgresProvince::new(database.clone(), logger.clone());
    let repository = PostgresCity::new(database, logger.clone());
    let context = context();

    let province = provinces
        .create(&context, province_create("11", "Aceh"))
        .await
        .expect("create parent province");
    let created = repository
        .create(
            &context,
            city_create(province.id, "11.01", "Simeulue", CityKind::Regency),
        )
        .await
        .expect("create city");
    assert_eq!(created.kind, CityKind::Regency);
    assert_eq!(created.province_id, province.id);
    assert_boundaries_close(&created.boundary, &boundary());

    assert_eq!(
        repository
            .find_by_kemendagri_code(&context, "11.01")
            .await
            .expect("find city by code")
            .expect("city exists")
            .id,
        created.id
    );
    assert!(matches!(
        repository
            .create(
                &context,
                city_create(province.id, "11.01", "Duplicate", CityKind::City)
            )
            .await,
        Err(RepositoryError::CityKemendagriCodeConflict)
    ));
    assert!(matches!(
        repository
            .create(
                &context,
                city_create(i64::MAX, "11.02", "Missing", CityKind::City)
            )
            .await,
        Err(RepositoryError::CityProvinceNotFound)
    ));

    repository
        .create(
            &context,
            city_create(province.id, "11.02", "Banda Aceh", CityKind::City),
        )
        .await
        .expect("create second city");
    let filtered = repository
        .list(
            &context,
            CityListArgs {
                province_id: Some(province.id),
                kind: Some(CityKind::City),
                is_active: Some(true),
                name_contains: Some("banda".into()),
                include_deleted: false,
                pagination: pagination(10, 0),
            },
        )
        .await
        .expect("filter cities");
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].kemendagri_code, "11.02");

    let updated = repository
        .update(
            &context,
            created.id,
            city_contract::UpdateArgs {
                kind: Some(CityKind::City),
                name: Some("Simeulue City".into()),
                boundary: Some(boundary()),
                boundary_source: Some("updated source".into()),
                boundary_reference_date: Some(None),
                is_active: Some(false),
            },
        )
        .await
        .expect("update city");
    assert_eq!(updated.kind, CityKind::City);
    assert_eq!(updated.name, "Simeulue City");
    assert_eq!(updated.boundary_reference_date, None);
    assert!(!updated.is_active);
    assert!(updated.updated_at.is_some());

    let unchanged = repository
        .update(
            &context,
            created.id,
            city_contract::UpdateArgs {
                kind: None,
                name: None,
                boundary: None,
                boundary_source: None,
                boundary_reference_date: None,
                is_active: None,
            },
        )
        .await
        .expect("no-op update city");
    assert_eq!(unchanged.updated_at, updated.updated_at);

    repository
        .delete(&context, created.id)
        .await
        .expect("delete city");
    assert!(
        repository
            .find_by_id(&context, created.id)
            .await
            .expect("find deleted city")
            .expect("deleted city remains readable")
            .deleted_at
            .is_some()
    );
    assert_eq!(
        repository
            .list(
                &context,
                CityListArgs {
                    province_id: None,
                    kind: None,
                    is_active: None,
                    name_contains: None,
                    include_deleted: false,
                    pagination: pagination(10, 0),
                },
            )
            .await
            .expect("list visible cities")
            .len(),
        1
    );
    assert_eq!(
        repository
            .list(
                &context,
                CityListArgs {
                    province_id: None,
                    kind: None,
                    is_active: None,
                    name_contains: None,
                    include_deleted: true,
                    pagination: pagination(10, 0),
                },
            )
            .await
            .expect("list all cities")
            .len(),
        2
    );
    assert!(matches!(
        repository
            .update(
                &context,
                created.id,
                city_contract::UpdateArgs {
                    kind: None,
                    name: Some("Hidden".into()),
                    boundary: None,
                    boundary_source: None,
                    boundary_reference_date: None,
                    is_active: None,
                }
            )
            .await,
        Err(RepositoryError::CityNotFound)
    ));

    fixture.close().await;
}

#[tokio::test]
async fn cities_can_reference_soft_deleted_provinces() {
    let Some(fixture) = Fixture::new().await else {
        return;
    };
    let logger = Arc::new(CapturingLogger::default());
    let database = fixture.database();
    let provinces = PostgresProvince::new(database.clone(), logger.clone());
    let cities = PostgresCity::new(database, logger);
    let context = context();

    let province = provinces
        .create(&context, province_create("11", "Aceh"))
        .await
        .expect("create parent province");
    provinces
        .delete(&context, province.id)
        .await
        .expect("soft delete parent province");

    let city = cities
        .create(
            &context,
            city_create(province.id, "11.01", "Simeulue", CityKind::Regency),
        )
        .await
        .expect("reference soft-deleted province");

    assert_eq!(city.province_id, province.id);

    fixture.close().await;
}
