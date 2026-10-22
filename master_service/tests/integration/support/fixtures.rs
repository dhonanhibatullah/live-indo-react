use chrono::NaiveDate;
use master_service::domain::{
    contracts::repository::{
        Pagination, city as city_contract, province as province_contract, topic as topic_contract,
    },
    models::{
        AppContext,
        entity::{Boundary, CityKind},
    },
};
use pgdt::sqlx;

use super::database::Fixture;

pub(crate) fn context() -> AppContext {
    AppContext {
        actor: Some("anonymous".into()),
        ..AppContext::default()
    }
}

pub(crate) fn boundary() -> Boundary {
    Boundary::new(vec![vec![vec![
        [106.0, -6.0],
        [107.0, -6.0],
        [107.0, -7.0],
        [106.0, -6.0],
    ]]])
    .expect("test boundary must be valid")
}

pub(crate) fn assert_boundaries_close(actual: &Boundary, expected: &Boundary) {
    let actual = actual.coordinates();
    let expected = expected.coordinates();
    assert_eq!(actual.len(), expected.len());
    for (actual_polygon, expected_polygon) in actual.iter().zip(expected) {
        assert_eq!(actual_polygon.len(), expected_polygon.len());
        for (actual_ring, expected_ring) in actual_polygon.iter().zip(expected_polygon) {
            assert_eq!(actual_ring.len(), expected_ring.len());
            for (actual_position, expected_position) in actual_ring.iter().zip(expected_ring) {
                assert!((actual_position[0] - expected_position[0]).abs() < 1e-9);
                assert!((actual_position[1] - expected_position[1]).abs() < 1e-9);
            }
        }
    }
}

pub(crate) fn pagination(limit: u32, offset: u32) -> Pagination {
    Pagination { limit, offset }
}

pub(crate) fn topic_create(
    slug: &str,
    name: &str,
    polling_method_id: i64,
) -> topic_contract::CreateArgs {
    topic_contract::CreateArgs {
        slug: slug.into(),
        name: name.into(),
        description: format!("{name} description"),
        polling_method_id,
        selection_options: vec!["Setuju".into(), "Tidak setuju".into()],
        is_active: true,
    }
}

pub(crate) fn topic_update() -> topic_contract::UpdateArgs {
    topic_contract::UpdateArgs {
        name: Some("Updated topic".into()),
        description: Some("Updated description".into()),
        polling_method_id: None,
        selection_options: Some(vec!["Yes".into(), "No".into()]),
        is_active: Some(false),
    }
}

pub(crate) fn province_create(code: &str, name: &str) -> province_contract::CreateArgs {
    province_contract::CreateArgs {
        kemendagri_code: code.into(),
        name: name.into(),
        boundary: boundary(),
        boundary_source: "test source".into(),
        boundary_reference_date: Some(
            NaiveDate::from_ymd_opt(2025, 1, 1).expect("valid reference date"),
        ),
        is_active: true,
    }
}

pub(crate) fn city_create(
    province_id: i64,
    code: &str,
    name: &str,
    kind: CityKind,
) -> city_contract::CreateArgs {
    city_contract::CreateArgs {
        province_id,
        kemendagri_code: code.into(),
        kind,
        name: name.into(),
        boundary: boundary(),
        boundary_source: "test source".into(),
        boundary_reference_date: Some(
            NaiveDate::from_ymd_opt(2025, 1, 1).expect("valid reference date"),
        ),
        is_active: true,
    }
}

pub(crate) async fn seed_polling_methods(fixture: &Fixture) {
    for (code, name, is_active) in [
        ("agree_disagree", "Agree or disagree", true),
        ("selection", "Selection", true),
        ("retired", "Retired", false),
    ] {
        sqlx::query(
            "INSERT INTO polling_methods (code, name, description, is_active) VALUES ($1, $2, '', $3)",
        )
        .bind(code)
        .bind(name)
        .bind(is_active)
        .execute(fixture.pool())
        .await
        .expect("seed polling method");
    }
}
