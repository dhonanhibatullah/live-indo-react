use std::sync::{Arc, Mutex};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{DateTime, Utc};
use master_service::{
    application::misc::AppContextService,
    domain::{
        models::{AppContext, RepositoryError, entity::Topic},
        usecases::{
            UsecaseFuture,
            geographic::{GetCity, GetProvince, GetRegion, IsInsideIndonesia, Region},
            management::{
                city::{CreateCity, ManageCity, UpdateCity},
                indonesia_boundary::{ManageIndonesiaBoundary, UpdateIndonesiaBoundary},
                polling_method::{ManagePollingMethod, UpdatePollingMethod},
                province::{CreateProvince, ManageProvince, UpdateProvince},
                topic::{CreateTopic, ManageTopic, UpdateTopic},
            },
            topic::{ListPollingMethods, ListTopics, PollingMethodCatalog, TopicCatalog},
        },
    },
    infrastructure::utility::UuidAppContextFactory,
    presentation::http::{Dependencies, routes::router},
};
use tower::ServiceExt;
use uuid::Uuid;

struct FakeUsecases {
    topic: Topic,
    captured: Mutex<Option<AppContext>>,
}

impl FakeUsecases {
    fn new() -> Self {
        Self {
            topic: topic(),
            captured: Mutex::new(None),
        }
    }
}

impl ManageTopic for FakeUsecases {
    fn create_topic<'a>(
        &'a self,
        context: &'a AppContext,
        _: CreateTopic,
    ) -> UsecaseFuture<'a, Topic> {
        *self.captured.lock().expect("captured lock") = Some(context.clone());
        let topic = self.topic.clone();
        Box::pin(async move { Ok(topic) })
    }

    fn update_topic<'a>(&'a self, _: &'a AppContext, _: UpdateTopic) -> UsecaseFuture<'a, Topic> {
        unavailable()
    }

    fn delete_topic<'a>(&'a self, _: &'a AppContext, _: i64) -> UsecaseFuture<'a, ()> {
        unavailable()
    }
}

impl ManagePollingMethod for FakeUsecases {
    fn update_polling_method<'a>(
        &'a self,
        _: &'a AppContext,
        _: UpdatePollingMethod,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::PollingMethod> {
        unavailable()
    }

    fn delete_polling_method<'a>(&'a self, _: &'a AppContext, _: i64) -> UsecaseFuture<'a, ()> {
        unavailable()
    }
}

impl ManageProvince for FakeUsecases {
    fn create_province<'a>(
        &'a self,
        _: &'a AppContext,
        _: CreateProvince,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::Province> {
        unavailable()
    }

    fn update_province<'a>(
        &'a self,
        _: &'a AppContext,
        _: UpdateProvince,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::Province> {
        unavailable()
    }

    fn delete_province<'a>(&'a self, _: &'a AppContext, _: i64) -> UsecaseFuture<'a, ()> {
        unavailable()
    }
}

impl ManageCity for FakeUsecases {
    fn create_city<'a>(
        &'a self,
        _: &'a AppContext,
        _: CreateCity,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::City> {
        unavailable()
    }

    fn update_city<'a>(
        &'a self,
        _: &'a AppContext,
        _: UpdateCity,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::City> {
        unavailable()
    }

    fn delete_city<'a>(&'a self, _: &'a AppContext, _: i64) -> UsecaseFuture<'a, ()> {
        unavailable()
    }
}

impl ManageIndonesiaBoundary for FakeUsecases {
    fn update_indonesia_boundary<'a>(
        &'a self,
        _: &'a AppContext,
        _: UpdateIndonesiaBoundary,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::IndonesiaBoundary> {
        unavailable()
    }
}

impl GetProvince for FakeUsecases {
    fn get_province_by_id<'a>(
        &'a self,
        _: &'a AppContext,
        _: i64,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::Province> {
        unavailable()
    }

    fn get_province_by_kemendagri_code<'a>(
        &'a self,
        _: &'a AppContext,
        _: &'a str,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::Province> {
        unavailable()
    }
}

impl GetCity for FakeUsecases {
    fn get_city_by_id<'a>(
        &'a self,
        _: &'a AppContext,
        _: i64,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::City> {
        unavailable()
    }

    fn get_city_by_kemendagri_code<'a>(
        &'a self,
        _: &'a AppContext,
        _: &'a str,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::City> {
        unavailable()
    }
}

impl GetRegion for FakeUsecases {
    fn get_region_by_ip<'a>(
        &'a self,
        _: &'a AppContext,
        _: std::net::IpAddr,
    ) -> UsecaseFuture<'a, master_service::domain::usecases::geographic::RegionByIp> {
        unavailable()
    }

    fn get_region_by_coord<'a>(
        &'a self,
        _: &'a AppContext,
        _: f64,
        _: f64,
    ) -> UsecaseFuture<'a, Option<Region>> {
        unavailable()
    }
}

impl IsInsideIndonesia for FakeUsecases {
    fn is_inside_indonesia<'a>(
        &'a self,
        _: &'a AppContext,
        _: f64,
        _: f64,
    ) -> UsecaseFuture<'a, bool> {
        Box::pin(async { Ok(true) })
    }
}

impl PollingMethodCatalog for FakeUsecases {
    fn list_available_polling_methods<'a>(
        &'a self,
        _: &'a AppContext,
        _: ListPollingMethods,
    ) -> UsecaseFuture<'a, Vec<master_service::domain::models::entity::PollingMethod>> {
        unavailable()
    }

    fn get_polling_method_by_code<'a>(
        &'a self,
        _: &'a AppContext,
        _: &'a str,
    ) -> UsecaseFuture<'a, master_service::domain::models::entity::PollingMethod> {
        unavailable()
    }
}

impl TopicCatalog for FakeUsecases {
    fn list_active_topics<'a>(
        &'a self,
        _: &'a AppContext,
        _: ListTopics,
    ) -> UsecaseFuture<'a, Vec<Topic>> {
        unavailable()
    }

    fn get_topic_by_slug<'a>(&'a self, _: &'a AppContext, _: &'a str) -> UsecaseFuture<'a, Topic> {
        Box::pin(async { Err(RepositoryError::TopicNotFound.into()) })
    }
}

fn unavailable<T: Send + 'static>() -> UsecaseFuture<'static, T> {
    Box::pin(async { Err(RepositoryError::Unimplemented.into()) })
}

fn topic() -> Topic {
    Topic {
        id: 1,
        slug: "topic".into(),
        name: "Topic".into(),
        description: String::new(),
        polling_method_id: 1,
        selection_options: Vec::new(),
        is_active: true,
        created_at: DateTime::<Utc>::from_timestamp(0, 0).expect("timestamp"),
        updated_at: None,
        deleted_at: None,
    }
}

fn dependencies(fake: Arc<FakeUsecases>) -> Dependencies {
    Dependencies {
        create_app_context: Arc::new(AppContextService::new(Arc::new(
            UuidAppContextFactory::new(),
        ))),
        manage_topic: fake.clone(),
        manage_polling_method: fake.clone(),
        manage_province: fake.clone(),
        manage_city: fake.clone(),
        manage_indonesia_boundary: fake.clone(),
        get_province: fake.clone(),
        get_city: fake.clone(),
        get_region: fake.clone(),
        is_inside_indonesia: fake.clone(),
        polling_method_catalog: fake.clone(),
        topic_catalog: fake,
    }
}

fn request(method: &str, uri: &str, body: Option<&str>, trace: bool) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if body.is_some() {
        builder = builder.header("content-type", "application/json");
    }
    if trace {
        builder = builder
            .header("x-trace-id", "00000000-0000-0000-0000-00000000002a")
            .header("x-actor", "actor");
    }
    builder
        .body(body.map_or_else(Body::empty, |body| Body::from(body.to_owned())))
        .expect("request")
}

#[tokio::test]
async fn http_management_uses_metadata_context_and_returns_created_entity() {
    let fake = Arc::new(FakeUsecases::new());
    let response = router(dependencies(fake.clone()))
        .oneshot(request(
            "POST",
            "/v1/topics",
            Some(r#"{"slug":"created","name":"Created","description":"","polling_method_id":1,"selection_options":[],"is_active":true}"#),
            true,
        ))
        .await
        .expect("route response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let context = fake
        .captured
        .lock()
        .expect("captured lock")
        .clone()
        .expect("captured context");
    assert_eq!(context.actor.as_deref(), Some("actor"));
    assert_eq!(context.trace_id, Some(Uuid::from_u128(42)));
    assert!(context.execution_id.is_some_and(|id| !id.is_nil()));

    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    let body: serde_json::Value = serde_json::from_slice(&body).expect("json response");
    assert_eq!(body["slug"], "topic");
}

#[tokio::test]
async fn http_rejects_missing_metadata_and_maps_domain_errors() {
    let fake = Arc::new(FakeUsecases::new());
    let app = router(dependencies(fake));

    let response = app
        .clone()
        .oneshot(request("GET", "/v1/topics/by-slug/topic", None, false))
        .await
        .expect("missing metadata response");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    let body: serde_json::Value = serde_json::from_slice(&body).expect("json response");
    assert_eq!(body["code"], "BAD_ARGS");

    let response = app
        .oneshot(request("GET", "/v1/topics/by-slug/topic", None, true))
        .await
        .expect("domain error response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body");
    let body: serde_json::Value = serde_json::from_slice(&body).expect("json response");
    assert_eq!(body["code"], "TOPIC_NOT_FOUND");
}

#[tokio::test]
async fn http_ip_region_lookup_preserves_estimates_absence_and_context_for_both_ip_families() {
    use crate::support::region::{Mode, RegionUsecase};
    let regions = Arc::new(RegionUsecase::new());
    let mut dependencies = dependencies(Arc::new(FakeUsecases::new()));
    dependencies.get_region = regions.clone();
    let app = router(dependencies);
    for ip in ["203.0.113.1", "2001:db8::1"] {
        for mode in [
            Mode::Full,
            Mode::EstimateOnly,
            Mode::NoCoordinates,
            Mode::Unknown,
        ] {
            regions.set(mode);
            let response = app
                .clone()
                .oneshot(request(
                    "GET",
                    &format!("/v1/regions/by-ip?ip={ip}"),
                    None,
                    true,
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            assert!(!String::from_utf8_lossy(&bytes).contains(ip));
            assert!(body.get("ip").is_none());
            if matches!(mode, Mode::Unknown) {
                assert!(body["geoip"].is_null());
            } else {
                assert_eq!(body["geoip"]["country_code"], "ID");
                assert_eq!(body["geoip"]["subdivisions"][0]["code"], "JK");
                assert_eq!(body["geoip"]["accuracy_radius_km"], 10);
                assert_eq!(body["geoip"]["time_zone"], "Asia/Jakarta");
                assert!(body["geoip"]["postal_code"].is_null());
                if matches!(mode, Mode::NoCoordinates) {
                    assert!(body["geoip"]["coordinates"].is_null());
                } else {
                    assert_eq!(body["geoip"]["coordinates"]["longitude"], 106.8);
                }
            }
            if matches!(mode, Mode::Full) {
                assert_eq!(body["region"]["province"]["kemendagri_code"], "31");
                assert_eq!(body["region"]["city"]["kemendagri_code"], "31.01");
            } else {
                assert!(body["region"].is_null());
            }
            let calls = regions.calls.lock().unwrap();
            let (context, observed) = calls.last().unwrap();
            assert_eq!(*observed, ip.parse::<std::net::IpAddr>().unwrap());
            assert_eq!(context.actor.as_deref(), Some("actor"));
            assert_eq!(context.trace_id, Some(Uuid::from_u128(42)));
            assert!(context.execution_id.is_some());
        }
    }
    let coordinate = app
        .oneshot(request(
            "GET",
            "/v1/regions/by-coordinate?longitude=106.8&latitude=-6.2",
            None,
            true,
        ))
        .await
        .unwrap();
    assert_eq!(coordinate.status(), StatusCode::OK);
}

#[tokio::test]
async fn http_ip_lookup_rejects_invalid_inputs_without_calling_usecase_or_echoing_ip() {
    use crate::support::region::RegionUsecase;
    let regions = Arc::new(RegionUsecase::new());
    let mut dependencies = dependencies(Arc::new(FakeUsecases::new()));
    dependencies.get_region = regions.clone();
    let app = router(dependencies);
    for query in [
        "",
        "?ip=",
        "?ip=not-an-ip",
        "?ip=203.0.113.1:80",
        "?ip=203.0.113.1/24",
        "?ip=203.0.113.1&extra=true",
        "?ip=203.0.113.1&ip=203.0.113.2",
    ] {
        let response = app
            .clone()
            .oneshot(request(
                "GET",
                &format!("/v1/regions/by-ip{query}"),
                None,
                true,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(body["code"], "BAD_ARGS");
        assert_eq!(body["message"], "invalid query parameter");
    }
    let missing_context = app
        .oneshot(request(
            "GET",
            "/v1/regions/by-ip?ip=203.0.113.1",
            None,
            false,
        ))
        .await
        .unwrap();
    assert_eq!(missing_context.status(), StatusCode::BAD_REQUEST);
    assert!(regions.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn http_ip_lookup_maps_classified_geoip_errors_to_safe_responses() {
    use crate::support::region::{Failure, Mode, RegionUsecase};
    let regions = Arc::new(RegionUsecase::new());
    let mut dependencies = dependencies(Arc::new(FakeUsecases::new()));
    dependencies.get_region = regions.clone();
    let app = router(dependencies);
    for (failure, status) in [
        (Failure::Configuration, StatusCode::INTERNAL_SERVER_ERROR),
        (Failure::NotReady, StatusCode::SERVICE_UNAVAILABLE),
        (Failure::Database, StatusCode::INTERNAL_SERVER_ERROR),
        (Failure::Download, StatusCode::INTERNAL_SERVER_ERROR),
        (Failure::Unavailable, StatusCode::SERVICE_UNAVAILABLE),
        (Failure::Timeout, StatusCode::GATEWAY_TIMEOUT),
        (Failure::Operation, StatusCode::INTERNAL_SERVER_ERROR),
    ] {
        regions.set(Mode::Error(failure));
        let response = app
            .clone()
            .oneshot(request(
                "GET",
                "/v1/regions/by-ip?ip=203.0.113.1",
                None,
                true,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["code"], failure.error().code());
        assert!(!String::from_utf8_lossy(&bytes).contains("private-provider-secret"));
        assert!(!String::from_utf8_lossy(&bytes).contains("203.0.113.1"));
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            assert_eq!(body["message"], "something went wrong");
        }
    }
}
