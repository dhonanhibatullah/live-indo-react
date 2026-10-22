use chrono::{DateTime, Utc};
use master_service::domain::{
    contracts::repository::{
        Pagination, city as city_contract, polling_method as polling_contract,
        province as province_contract, topic as topic_contract,
    },
    models::entity::{Boundary, City, CityKind, PollingMethod, Province, Topic},
};

pub(crate) fn timestamp() -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(0, 0).expect("valid test timestamp")
}

pub(crate) fn boundary() -> Boundary {
    Boundary::new(vec![vec![vec![
        [106.0, -6.0],
        [107.0, -6.0],
        [107.0, -7.0],
        [106.0, -6.0],
    ]]])
    .expect("valid test boundary")
}

pub(crate) fn pagination(limit: u32, offset: u32) -> Pagination {
    Pagination { limit, offset }
}

pub(crate) fn topic() -> Topic {
    Topic {
        id: 1,
        slug: "topic".into(),
        name: "Topic".into(),
        description: "Topic description".into(),
        polling_method_id: 1,
        selection_options: vec!["Yes".into(), "No".into()],
        is_active: true,
        created_at: timestamp(),
        updated_at: None,
        deleted_at: None,
    }
}

pub(crate) fn polling_method() -> PollingMethod {
    PollingMethod {
        id: 1,
        code: "agree_disagree".into(),
        name: "Agree or disagree".into(),
        description: "Polling method description".into(),
        is_active: true,
        created_at: timestamp(),
        updated_at: None,
        deleted_at: None,
    }
}

pub(crate) fn province() -> Province {
    Province {
        id: 1,
        kemendagri_code: "11".into(),
        name: "Aceh".into(),
        boundary: boundary(),
        boundary_source: "test source".into(),
        boundary_reference_date: None,
        is_active: true,
        created_at: timestamp(),
        updated_at: None,
        deleted_at: None,
    }
}

pub(crate) fn city() -> City {
    City {
        id: 1,
        province_id: 1,
        kemendagri_code: "11.01".into(),
        kind: CityKind::City,
        name: "City".into(),
        boundary: boundary(),
        boundary_source: "test source".into(),
        boundary_reference_date: None,
        is_active: true,
        created_at: timestamp(),
        updated_at: None,
        deleted_at: None,
    }
}

pub(crate) fn topic_create() -> topic_contract::CreateArgs {
    topic_contract::CreateArgs {
        slug: "created".into(),
        name: "Created".into(),
        description: "Created description".into(),
        polling_method_id: 1,
        selection_options: vec!["Yes".into(), "No".into()],
        is_active: true,
    }
}

pub(crate) fn topic_update() -> topic_contract::UpdateArgs {
    topic_contract::UpdateArgs {
        name: Some("Updated Topic".into()),
        description: None,
        polling_method_id: None,
        selection_options: None,
        is_active: None,
    }
}

pub(crate) fn polling_method_update() -> polling_contract::UpdateArgs {
    polling_contract::UpdateArgs {
        name: Some("Updated Method".into()),
        description: None,
        is_active: None,
    }
}

pub(crate) fn province_create() -> province_contract::CreateArgs {
    province_contract::CreateArgs {
        kemendagri_code: "12".into(),
        name: "Created Province".into(),
        boundary: boundary(),
        boundary_source: "test source".into(),
        boundary_reference_date: None,
        is_active: true,
    }
}

pub(crate) fn province_update() -> province_contract::UpdateArgs {
    province_contract::UpdateArgs {
        name: Some("Updated Province".into()),
        boundary: None,
        boundary_source: None,
        boundary_reference_date: None,
        is_active: None,
    }
}

pub(crate) fn city_create() -> city_contract::CreateArgs {
    city_contract::CreateArgs {
        province_id: 1,
        kemendagri_code: "11.02".into(),
        kind: CityKind::City,
        name: "Created City".into(),
        boundary: boundary(),
        boundary_source: "test source".into(),
        boundary_reference_date: None,
        is_active: true,
    }
}

pub(crate) fn city_update() -> city_contract::UpdateArgs {
    city_contract::UpdateArgs {
        kind: None,
        name: Some("Updated City".into()),
        boundary: None,
        boundary_source: None,
        boundary_reference_date: None,
        is_active: None,
    }
}
