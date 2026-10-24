use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Utc};
use master_service::{
    domain::{
        models::{AppContext, GeoIpError},
        usecases::{UsecaseFuture, maintenance::RefreshGeoIp, misc::app_context::CreateAppContext},
    },
    presentation::cron::{Runner, register::maintenance},
};
use tokio::{
    sync::{Notify, watch},
    time::{Instant, advance},
};
use uuid::Uuid;

#[derive(Default)]
struct ContextFactory(AtomicU64);
impl CreateAppContext for ContextFactory {
    fn create_app_context(&self, actor: Option<String>, trace_id: Option<Uuid>) -> AppContext {
        AppContext {
            actor,
            trace_id,
            execution_id: Some(Uuid::from_u128(
                u128::from(self.0.fetch_add(1, Ordering::Relaxed)) + 1,
            )),
            transaction: None,
        }
    }
}
#[derive(Default)]
struct Refresh {
    calls: Mutex<Vec<AppContext>>,
    gate: Option<Arc<Notify>>,
    fail: bool,
}
impl RefreshGeoIp for Refresh {
    fn refresh_geoip<'a>(&'a self, context: &'a AppContext) -> UsecaseFuture<'a, ()> {
        Box::pin(async move {
            self.calls.lock().unwrap().push(context.clone());
            if let Some(gate) = &self.gate {
                gate.notified().await;
            }
            if self.fail {
                return Err(GeoIpError::Unavailable.into());
            }
            Ok(())
        })
    }
}
fn runner(refresh: Arc<Refresh>, execution_timeout: Duration) -> Runner {
    let start = Instant::now();
    let wall: SystemTime = "2026-10-24T23:59:50Z"
        .parse::<DateTime<Utc>>()
        .unwrap()
        .into();
    let mut jobs = maintenance::jobs(refresh);
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].execution_timeout, Duration::from_secs(600));
    jobs[0].execution_timeout = execution_timeout;
    Runner::new(Arc::new(ContextFactory::default()), jobs)
        .with_clock(move || wall + start.elapsed())
}
async fn tick(seconds: u64) {
    advance(Duration::from_secs(seconds)).await;
    // Allow the runner and its worker to react without wall-clock sleeps.
    tokio::task::yield_now().await;
    tokio::task::yield_now().await;
}

#[tokio::test(start_paused = true)]
async fn daily_registration_waits_until_midnight_and_creates_fresh_contexts() {
    let refresh = Arc::new(Refresh::default());
    let runner = runner(refresh.clone(), Duration::from_secs(600));
    let (shutdown, receiver) = watch::channel(false);
    let task = tokio::spawn(runner.run(receiver));
    tokio::task::yield_now().await;
    tick(9).await;
    assert!(refresh.calls.lock().unwrap().is_empty());
    tick(1).await;
    assert_eq!(refresh.calls.lock().unwrap().len(), 1);
    tick(86400).await;
    let calls = refresh.calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 2);
    assert!(calls.iter().all(|context| context.actor.as_deref()
        == Some("cron/maintenance.geoip.refresh")
        && context.transaction.is_none()
        && context.trace_id.is_none()));
    assert_ne!(calls[0].execution_id, calls[1].execution_id);
    shutdown.send(true).unwrap();
    task.await.unwrap().unwrap();
}

#[tokio::test(start_paused = true)]
async fn failed_executions_wait_for_the_next_day_instead_of_stopping_the_runner() {
    let refresh = Arc::new(Refresh {
        fail: true,
        ..Refresh::default()
    });
    let (shutdown, receiver) = watch::channel(false);
    let task = tokio::spawn(runner(refresh.clone(), Duration::from_secs(600)).run(receiver));
    tokio::task::yield_now().await;
    tick(10).await;
    tick(86400).await;
    assert_eq!(refresh.calls.lock().unwrap().len(), 2);
    shutdown.send(true).unwrap();
    task.await.unwrap().unwrap();
}

#[tokio::test(start_paused = true)]
async fn slow_executions_do_not_overlap_or_replay_missed_days_and_shutdown_drains_them() {
    let gate = Arc::new(Notify::new());
    let refresh = Arc::new(Refresh {
        gate: Some(gate.clone()),
        ..Refresh::default()
    });
    let (shutdown, receiver) = watch::channel(false);
    let task = tokio::spawn(runner(refresh.clone(), Duration::from_secs(3 * 86400)).run(receiver));
    tokio::task::yield_now().await;
    tick(10).await;
    tick(86400).await;
    assert_eq!(refresh.calls.lock().unwrap().len(), 1);
    gate.notify_one();
    tokio::task::yield_now().await;
    tick(1).await;
    assert_eq!(refresh.calls.lock().unwrap().len(), 1);
    tick(86399).await;
    assert_eq!(refresh.calls.lock().unwrap().len(), 2);
    shutdown.send(true).unwrap();
    tokio::task::yield_now().await;
    assert!(!task.is_finished());
    gate.notify_one();
    task.await.unwrap().unwrap();
}

#[tokio::test(start_paused = true)]
async fn timed_out_runs_are_cancelled_without_a_retry_before_the_next_daily_trigger() {
    let gate = Arc::new(Notify::new());
    let refresh = Arc::new(Refresh {
        gate: Some(gate.clone()),
        ..Refresh::default()
    });
    let (shutdown, receiver) = watch::channel(false);
    let task = tokio::spawn(runner(refresh.clone(), Duration::from_secs(5)).run(receiver));
    tokio::task::yield_now().await;
    tick(10).await;
    tick(5).await;
    assert_eq!(refresh.calls.lock().unwrap().len(), 1);
    tick(86395).await;
    assert_eq!(refresh.calls.lock().unwrap().len(), 2);
    shutdown.send(true).unwrap();
    gate.notify_one();
    task.await.unwrap().unwrap();
}

#[tokio::test(start_paused = true)]
async fn shutdown_before_the_first_trigger_prevents_job_execution() {
    let refresh = Arc::new(Refresh::default());
    let (shutdown, receiver) = watch::channel(true);
    runner(refresh.clone(), Duration::from_secs(600))
        .run(receiver)
        .await
        .unwrap();
    drop(shutdown);
    assert!(refresh.calls.lock().unwrap().is_empty());
}
