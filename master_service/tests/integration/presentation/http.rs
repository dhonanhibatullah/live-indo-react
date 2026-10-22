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
            geographic::{GetCity, GetProvince, GetRegionByCoord, IsInsideIndonesia, Region},
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

impl GetRegionByCoord for FakeUsecases {
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
