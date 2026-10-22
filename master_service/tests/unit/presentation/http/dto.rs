use chrono::NaiveDate;
use master_service::{
    domain::usecases::management::{
        province::{CreateProvince, UpdateProvince},
        topic::UpdateTopic,
    },
    presentation::http::dto::request::management::{
        province::{CreateProvinceRequest, UpdateProvinceRequest},
        topic::UpdateTopicRequest,
    },
};

fn boundary_json() -> serde_json::Value {
    serde_json::json!({
        "type": "MultiPolygon",
        "coordinates": [[[[106.0, -6.0], [107.0, -6.0], [107.0, -7.0], [106.0, -6.0]]]]
    })
}

#[test]
fn management_dtos_preserve_date_and_selection_presence() {
    let update: UpdateTopic = serde_json::from_value::<UpdateTopicRequest>(serde_json::json!({
        "selection_options": []
    }))
    .expect("update topic dto")
    .into();
    assert_eq!(update.selection_options, Some(Vec::new()));

    let update: UpdateProvince =
        serde_json::from_value::<UpdateProvinceRequest>(serde_json::json!({
            "clear_boundary_reference_date": true
        }))
        .expect("update province dto")
        .try_into()
        .expect("update province conversion");
    assert_eq!(update.boundary_reference_date, Some(None));

    let create: CreateProvince =
        serde_json::from_value::<CreateProvinceRequest>(serde_json::json!({
            "kemendagri_code": "11",
            "name": "Aceh",
            "boundary": boundary_json(),
            "boundary_source": "test",
            "boundary_reference_date": "2026-10-22",
            "is_active": true
        }))
        .expect("create province dto")
        .try_into()
        .expect("create province conversion");
    assert_eq!(
        create.boundary_reference_date,
        Some(NaiveDate::from_ymd_opt(2026, 10, 22).expect("date"))
    );
}

#[test]
fn invalid_boundary_and_date_are_rejected() {
    let invalid = serde_json::from_value::<CreateProvinceRequest>(serde_json::json!({
        "kemendagri_code": "11",
        "name": "Aceh",
        "boundary": {"type": "MultiPolygon", "coordinates": []},
        "boundary_source": "test",
        "is_active": true
    }))
    .expect("boundary shape");
    assert!(TryInto::<CreateProvince>::try_into(invalid).is_err());

    let invalid_date = serde_json::from_value::<UpdateProvinceRequest>(serde_json::json!({
        "boundary_reference_date": "not-a-date"
    }))
    .expect("date shape");
    assert!(TryInto::<UpdateProvince>::try_into(invalid_date).is_err());
}
