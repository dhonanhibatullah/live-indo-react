use std::{
    collections::VecDeque,
    error::Error,
    io,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use master_service::{
    application::maintenance::GeoIpMaintenance,
    domain::{
        contracts::service::{GeoIpFuture, GeoIpService},
        models::{AppContext, AppTransaction, GeoIpError, entity::GeoIp, logger::Level},
        usecases::maintenance::RefreshGeoIp,
    },
};

use super::support::CapturingLogger;

// A strict script makes unexpected extra checks, updates or reordered calls fail.
enum Step {
    Ready(Result<bool, GeoIpError>),
    Check(Result<bool, GeoIpError>),
    Update(Result<(), GeoIpError>),
}
struct ScriptedGeoIp {
    steps: Mutex<VecDeque<Step>>,
    contexts: Mutex<Vec<AppContext>>,
}
impl ScriptedGeoIp {
    fn new(steps: Vec<Step>) -> Self {
        Self {
            steps: Mutex::new(steps.into()),
            contexts: Mutex::new(Vec::new()),
        }
    }
    fn next(&self, context: &AppContext) -> Step {
        self.contexts.lock().unwrap().push(context.clone());
        self.steps
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected GeoIP operation")
    }
    fn verify(&self, context: &AppContext, calls: usize) {
        assert!(self.steps.lock().unwrap().is_empty());
        let contexts = self.contexts.lock().unwrap();
        assert_eq!(contexts.len(), calls);
        assert!(contexts.iter().all(|observed| observed == context));
    }
}
impl GeoIpService for ScriptedGeoIp {
    fn is_ready<'a>(&'a self, context: &'a AppContext) -> GeoIpFuture<'a, bool> {
        let Step::Ready(result) = self.next(context) else {
            panic!("expected readiness check");
        };
        Box::pin(async move { result })
    }
    fn check_for_update<'a>(&'a self, context: &'a AppContext) -> GeoIpFuture<'a, bool> {
        let Step::Check(result) = self.next(context) else {
            panic!("expected update check");
        };
        Box::pin(async move { result })
    }
    fn update<'a>(&'a self, context: &'a AppContext) -> GeoIpFuture<'a, ()> {
        let Step::Update(result) = self.next(context) else {
            panic!("expected update");
        };
        Box::pin(async move { result })
    }
    fn get_location<'a>(&'a self, _: &'a AppContext, _: IpAddr) -> GeoIpFuture<'a, Option<GeoIp>> {
        panic!("maintenance must not look up a location")
    }
}
fn context() -> AppContext {
    AppContext {
        actor: Some("maintenance-test".into()),
        transaction: Some(AppTransaction::new("opaque-test-handle")),
        ..AppContext::default()
    }
}

#[tokio::test]
async fn unready_database_updates_without_a_separate_update_check() {
    let geoip = Arc::new(ScriptedGeoIp::new(vec![
        Step::Ready(Ok(false)),
        Step::Update(Ok(())),
    ]));
    let logger = Arc::new(CapturingLogger::default());
    let application = GeoIpMaintenance::new(geoip.clone(), logger.clone());
    let context = context();
    application.refresh_geoip(&context).await.unwrap();
    geoip.verify(&context, 2);
    assert!(logger.records().is_empty());
}

#[tokio::test]
async fn ready_database_updates_only_when_a_new_release_exists() {
    for newer in [false, true] {
        let mut steps = vec![Step::Ready(Ok(true)), Step::Check(Ok(newer))];
        if newer {
            steps.push(Step::Update(Ok(())));
        }
        let geoip = Arc::new(ScriptedGeoIp::new(steps));
        let logger = Arc::new(CapturingLogger::default());
        let application = GeoIpMaintenance::new(geoip.clone(), logger.clone());
        let context = context();
        application.refresh_geoip(&context).await.unwrap();
        geoip.verify(&context, if newer { 3 } else { 2 });
        let records = logger.records();
        if newer {
            assert!(records.is_empty());
        } else {
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].level, Level::Info);
            assert_eq!(records[0].tag, "maintenance/geoip/refresh");
            assert_eq!(
                records[0].message,
                "GeoIP refresh skipped: database is current"
            );
            assert!(records[0].meta.is_none());
        }
    }
}

#[tokio::test]
async fn each_failure_stops_later_operations_and_preserves_classification_and_native_cause() {
    let timeout = || GeoIpError::Timeout {
        source: Box::new(io::Error::other("private-native-cause")),
    };
    for steps in [
        vec![Step::Ready(Err(timeout()))],
        vec![Step::Ready(Ok(true)), Step::Check(Err(timeout()))],
        vec![Step::Ready(Ok(false)), Step::Update(Err(timeout()))],
        vec![
            Step::Ready(Ok(true)),
            Step::Check(Ok(true)),
            Step::Update(Err(timeout())),
        ],
    ] {
        let calls = steps.len();
        let geoip = Arc::new(ScriptedGeoIp::new(steps));
        let logger = Arc::new(CapturingLogger::default());
        let application = GeoIpMaintenance::new(geoip.clone(), logger.clone());
        let context = context();
        let error = application.refresh_geoip(&context).await.unwrap_err();
        assert_eq!(error.code(), "GEOIP_TIMEOUT");
        let category = error
            .source()
            .unwrap()
            .downcast_ref::<GeoIpError>()
            .unwrap();
        let native = category
            .source()
            .unwrap()
            .downcast_ref::<io::Error>()
            .unwrap();
        assert_eq!(native.to_string(), "private-native-cause");
        assert!(!format!("{error:?}").contains("private-native-cause"));
        geoip.verify(&context, calls);
        assert!(logger.records().is_empty());
    }
}
