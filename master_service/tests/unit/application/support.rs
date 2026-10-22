use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

use chrono::{DateTime, Utc};
use master_service::domain::{
    contracts::{
        repository::{
            CityRepository, IndonesiaBoundaryRepository, PollingMethodRepository,
            ProvinceRepository, RepositoryFuture, TopicRepository, city as city_contract,
            indonesia_boundary as boundary_contract, polling_method as polling_contract,
            province as province_contract, topic as topic_contract,
        },
        utility::{Logger, Validator},
    },
    models::{
        AppContext, RepositoryError, ValidatorError,
        entity::{Boundary, City, CityKind, IndonesiaBoundary, PollingMethod, Province, Topic},
        logger::{Level, Meta},
    },
};

fn timestamp() -> DateTime<Utc> {
    DateTime::<Utc>::from_timestamp(0, 0).expect("valid timestamp")
}

pub(crate) fn boundary() -> Boundary {
    Boundary::new(vec![vec![vec![
        [106.0, -6.0],
        [107.0, -6.0],
        [107.0, -7.0],
        [106.0, -6.0],
    ]]])
    .expect("valid boundary")
}

pub(crate) fn topic() -> Topic {
    Topic {
        id: 1,
        slug: "topic".into(),
        name: "Topic".into(),
        description: String::new(),
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
        code: "selection".into(),
        name: "Selection".into(),
        description: String::new(),
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
        boundary_source: "source".into(),
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
        kind: CityKind::Regency,
        name: "City".into(),
        boundary: boundary(),
        boundary_source: "source".into(),
        boundary_reference_date: None,
        is_active: true,
        created_at: timestamp(),
        updated_at: None,
        deleted_at: None,
    }
}

pub(crate) fn indonesia_boundary() -> IndonesiaBoundary {
    IndonesiaBoundary {
        id: 1,
        boundary: boundary(),
        boundary_source: "source".into(),
        boundary_reference_date: None,
        created_at: timestamp(),
        updated_at: None,
    }
}

pub(crate) struct TestValidator {
    invalid_slug: AtomicBool,
}

impl TestValidator {
    pub(crate) fn new() -> Self {
        Self {
            invalid_slug: AtomicBool::new(false),
        }
    }

    pub(crate) fn reject_slug(&self) {
        self.invalid_slug.store(true, Ordering::SeqCst);
    }
}

impl Validator for TestValidator {
    fn polling_method_code(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn polling_method_name(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn topic_slug(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        if self.invalid_slug.load(Ordering::SeqCst) {
            Err(ValidatorError::TopicSlugInvalid)
        } else {
            Ok(())
        }
    }
    fn topic_name(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn topic_selection_options(
        &self,
        _: &AppContext,
        _: &str,
        _: &[String],
    ) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn province_kemendagri_code(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn province_name(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn province_boundary_source(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn city_kemendagri_code(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn city_name(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn city_boundary_source(&self, _: &AppContext, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
    fn city_province_code(&self, _: &AppContext, _: &str, _: &str) -> Result<(), ValidatorError> {
        Ok(())
    }
}

pub(crate) struct FakeTopicRepository {
    topic: Topic,
    pub(crate) created: Mutex<Option<topic_contract::CreateArgs>>,
    pub(crate) updated: Mutex<Option<topic_contract::UpdateArgs>>,
    pub(crate) last_list: Mutex<Option<topic_contract::ListArgs>>,
}

impl FakeTopicRepository {
    pub(crate) fn new(topic: Topic) -> Self {
        Self {
            topic,
            created: Mutex::new(None),
            updated: Mutex::new(None),
            last_list: Mutex::new(None),
        }
    }
}

impl TopicRepository for FakeTopicRepository {
    fn create<'a>(
        &'a self,
        _: &'a AppContext,
        input: topic_contract::CreateArgs,
    ) -> RepositoryFuture<'a, Topic> {
        *self.created.lock().expect("created lock") = Some(input);
        let topic = self.topic.clone();
        Box::pin(async move { Ok(topic) })
    }

    fn find_by_id<'a>(&'a self, _: &'a AppContext, _: i64) -> RepositoryFuture<'a, Option<Topic>> {
        let topic = self.topic.clone();
        Box::pin(async move { Ok(Some(topic)) })
    }

    fn find_by_slug<'a>(
        &'a self,
        _: &'a AppContext,
        _: &'a str,
    ) -> RepositoryFuture<'a, Option<Topic>> {
        let topic = self.topic.clone();
        Box::pin(async move { Ok(Some(topic)) })
    }

    fn list<'a>(
        &'a self,
        _: &'a AppContext,
        filter: topic_contract::ListArgs,
    ) -> RepositoryFuture<'a, Vec<Topic>> {
        *self.last_list.lock().expect("list lock") = Some(filter);
        let topic = self.topic.clone();
        Box::pin(async move { Ok(vec![topic]) })
    }

    fn update<'a>(
        &'a self,
        _: &'a AppContext,
        _: i64,
        changes: topic_contract::UpdateArgs,
    ) -> RepositoryFuture<'a, Topic> {
        *self.updated.lock().expect("updated lock") = Some(changes);
        let topic = self.topic.clone();
        Box::pin(async move { Ok(topic) })
    }

    fn delete<'a>(&'a self, _: &'a AppContext, _: i64) -> RepositoryFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }
}

pub(crate) struct FakePollingMethodRepository {
    method: Mutex<Option<PollingMethod>>,
    pub(crate) last_list: Mutex<Option<polling_contract::ListArgs>>,
}

impl FakePollingMethodRepository {
    pub(crate) fn new(method: Option<PollingMethod>) -> Self {
        Self {
            method: Mutex::new(method),
            last_list: Mutex::new(None),
        }
    }

    pub(crate) fn clear(&self) {
        *self.method.lock().expect("method lock") = None;
    }
}

impl PollingMethodRepository for FakePollingMethodRepository {
    fn find_by_id<'a>(
        &'a self,
        _: &'a AppContext,
        _: i64,
    ) -> RepositoryFuture<'a, Option<PollingMethod>> {
        let method = self.method.lock().expect("method lock").clone();
        Box::pin(async move { Ok(method) })
    }

    fn find_by_code<'a>(
        &'a self,
        _: &'a AppContext,
        _: &'a str,
    ) -> RepositoryFuture<'a, Option<PollingMethod>> {
        let method = self.method.lock().expect("method lock").clone();
        Box::pin(async move { Ok(method) })
    }

    fn list<'a>(
        &'a self,
        _: &'a AppContext,
        filter: polling_contract::ListArgs,
    ) -> RepositoryFuture<'a, Vec<PollingMethod>> {
        *self.last_list.lock().expect("list lock") = Some(filter);
        let method = self.method.lock().expect("method lock").clone();
        Box::pin(async move { Ok(method.into_iter().collect()) })
    }

    fn update<'a>(
        &'a self,
        _: &'a AppContext,
        _: i64,
        _: polling_contract::UpdateArgs,
    ) -> RepositoryFuture<'a, PollingMethod> {
        Box::pin(async { Err(RepositoryError::Unimplemented) })
    }

    fn delete<'a>(&'a self, _: &'a AppContext, _: i64) -> RepositoryFuture<'a, ()> {
        Box::pin(async { Err(RepositoryError::Unimplemented) })
    }
}

pub(crate) struct FakeProvinceRepository {
    province: Province,
    pub(crate) by_coord: Mutex<Option<Province>>,
}

impl FakeProvinceRepository {
    pub(crate) fn new(province: Province) -> Self {
        Self {
            province,
            by_coord: Mutex::new(None),
        }
    }
}

impl ProvinceRepository for FakeProvinceRepository {
    fn create<'a>(
        &'a self,
        _: &'a AppContext,
        _: province_contract::CreateArgs,
    ) -> RepositoryFuture<'a, Province> {
        Box::pin(async { Err(RepositoryError::Unimplemented) })
    }
    fn find_by_id<'a>(
        &'a self,
        _: &'a AppContext,
        _: i64,
    ) -> RepositoryFuture<'a, Option<Province>> {
        let province = self.province.clone();
        Box::pin(async move { Ok(Some(province)) })
    }
    fn find_by_kemendagri_code<'a>(
        &'a self,
        _: &'a AppContext,
        _: &'a str,
    ) -> RepositoryFuture<'a, Option<Province>> {
        let province = self.province.clone();
        Box::pin(async move { Ok(Some(province)) })
    }
    fn find_province_by_coord<'a>(
        &'a self,
        _: &'a AppContext,
        _: f64,
        _: f64,
    ) -> RepositoryFuture<'a, Option<Province>> {
        let province = self.by_coord.lock().expect("coord lock").clone();
        Box::pin(async move { Ok(province) })
    }
    fn list<'a>(
        &'a self,
        _: &'a AppContext,
        _: province_contract::ListArgs,
    ) -> RepositoryFuture<'a, Vec<Province>> {
        Box::pin(async { Ok(Vec::new()) })
    }
    fn update<'a>(
        &'a self,
        _: &'a AppContext,
        _: i64,
        _: province_contract::UpdateArgs,
    ) -> RepositoryFuture<'a, Province> {
        Box::pin(async { Err(RepositoryError::Unimplemented) })
    }
    fn delete<'a>(&'a self, _: &'a AppContext, _: i64) -> RepositoryFuture<'a, ()> {
        Box::pin(async { Err(RepositoryError::Unimplemented) })
    }
}

pub(crate) struct FakeCityRepository {
    city: Mutex<Option<City>>,
}

impl FakeCityRepository {
    pub(crate) fn new(city: Option<City>) -> Self {
        Self {
            city: Mutex::new(city),
        }
    }
}

impl CityRepository for FakeCityRepository {
    fn create<'a>(
        &'a self,
        _: &'a AppContext,
        _: city_contract::CreateArgs,
    ) -> RepositoryFuture<'a, City> {
        Box::pin(async { Err(RepositoryError::Unimplemented) })
    }
    fn find_by_id<'a>(&'a self, _: &'a AppContext, _: i64) -> RepositoryFuture<'a, Option<City>> {
        let city = self.city.lock().expect("city lock").clone();
        Box::pin(async move { Ok(city) })
    }
    fn find_by_kemendagri_code<'a>(
        &'a self,
        _: &'a AppContext,
        _: &'a str,
    ) -> RepositoryFuture<'a, Option<City>> {
        let city = self.city.lock().expect("city lock").clone();
        Box::pin(async move { Ok(city) })
    }
    fn find_city_by_coord<'a>(
        &'a self,
        _: &'a AppContext,
        _: f64,
        _: f64,
    ) -> RepositoryFuture<'a, Option<City>> {
        let city = self.city.lock().expect("city lock").clone();
        Box::pin(async move { Ok(city) })
    }
    fn list<'a>(
        &'a self,
        _: &'a AppContext,
        _: city_contract::ListArgs,
    ) -> RepositoryFuture<'a, Vec<City>> {
        Box::pin(async { Ok(Vec::new()) })
    }
    fn update<'a>(
        &'a self,
        _: &'a AppContext,
        _: i64,
        _: city_contract::UpdateArgs,
    ) -> RepositoryFuture<'a, City> {
        Box::pin(async { Err(RepositoryError::Unimplemented) })
    }
    fn delete<'a>(&'a self, _: &'a AppContext, _: i64) -> RepositoryFuture<'a, ()> {
        Box::pin(async { Err(RepositoryError::Unimplemented) })
    }
}

pub(crate) struct FakeIndonesiaBoundaryRepository {
    pub(crate) inside: AtomicBool,
}

impl FakeIndonesiaBoundaryRepository {
    pub(crate) fn new(inside: bool) -> Self {
        Self {
            inside: AtomicBool::new(inside),
        }
    }
}

impl IndonesiaBoundaryRepository for FakeIndonesiaBoundaryRepository {
    fn get<'a>(&'a self, _: &'a AppContext) -> RepositoryFuture<'a, Option<IndonesiaBoundary>> {
        let boundary = indonesia_boundary();
        Box::pin(async move { Ok(Some(boundary)) })
    }
    fn update<'a>(
        &'a self,
        _: &'a AppContext,
        _: boundary_contract::UpdateArgs,
    ) -> RepositoryFuture<'a, IndonesiaBoundary> {
        let boundary = indonesia_boundary();
        Box::pin(async move { Ok(boundary) })
    }
    fn is_inside<'a>(&'a self, _: &'a AppContext, _: f64, _: f64) -> RepositoryFuture<'a, bool> {
        let inside = self.inside.load(Ordering::SeqCst);
        Box::pin(async move { Ok(inside) })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LogRecord {
    pub(crate) level: Level,
    pub(crate) tag: String,
    pub(crate) message: String,
    pub(crate) meta: Option<Meta>,
}

#[derive(Default)]
pub(crate) struct CapturingLogger {
    records: Mutex<Vec<LogRecord>>,
}

impl CapturingLogger {
    pub(crate) fn records(&self) -> Vec<LogRecord> {
        self.records.lock().expect("log lock").clone()
    }
}

impl Logger for CapturingLogger {
    fn log(&self, _: &AppContext, level: Level, tag: &str, message: &str, meta: Option<&Meta>) {
        self.records.lock().expect("log lock").push(LogRecord {
            level,
            tag: tag.to_owned(),
            message: message.to_owned(),
            meta: meta.cloned(),
        });
    }
}
