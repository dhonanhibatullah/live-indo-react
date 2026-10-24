use std::{
    error::Error,
    fmt,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, NaiveTime, Utc};
use tokio::{
    sync::watch,
    task::JoinSet,
    time::{sleep, timeout},
};

use crate::domain::usecases::misc::app_context::CreateAppContext;

use super::job::Job;

type Clock = Arc<dyn Fn() -> SystemTime + Send + Sync>;

/// Owns bounded daily workers; composition starts and joins this runner.
/// It deliberately has no logger. Application/infrastructure own operation logs.
pub struct Runner {
    create_app_context: Arc<dyn CreateAppContext>,
    jobs: Vec<Job>,
    clock: Clock,
}

impl Runner {
    pub fn new(create_app_context: Arc<dyn CreateAppContext>, jobs: Vec<Job>) -> Self {
        Self {
            create_app_context,
            jobs,
            clock: Arc::new(SystemTime::now),
        }
    }

    /// Inject a wall clock for deterministic scheduling tests or controlled clocks.
    /// Production uses SystemTime::now through new().
    pub fn with_clock(mut self, clock: impl Fn() -> SystemTime + Send + Sync + 'static) -> Self {
        self.clock = Arc::new(clock);
        self
    }

    /// Stop admitting runs when shutdown is signalled. In-flight calls drain until
    /// their execution timeout or composition's shorter shutdown deadline.
    pub async fn run(self, mut shutdown: watch::Receiver<bool>) -> Result<(), CronError> {
        let mut workers = JoinSet::new();
        for job in self.jobs {
            workers.spawn(run_job(
                job,
                self.create_app_context.clone(),
                shutdown.clone(),
                self.clock.clone(),
            ));
        }
        if workers.is_empty() {
            wait_for_shutdown(&mut shutdown).await;
        }
        while let Some(result) = workers.join_next().await {
            result.map_err(CronError::Worker)??;
        }
        Ok(())
    }
}

async fn run_job(
    job: Job,
    create_app_context: Arc<dyn CreateAppContext>,
    mut shutdown: watch::Receiver<bool>,
    clock: Clock,
) -> Result<(), CronError> {
    loop {
        if *shutdown.borrow() || shutdown.has_changed().is_err() {
            return Ok(());
        }
        let next = next_run(clock(), job.time_utc)?;
        loop {
            let now = clock();
            if now >= next {
                break;
            }
            let remaining = next.duration_since(now).map_err(|_| CronError::Clock)?;
            tokio::select! {
                biased;
                _ = wait_for_shutdown(&mut shutdown) => return Ok(()),
                _ = sleep(remaining.min(Duration::from_secs(60))) => {},
            }
        }
        if *shutdown.borrow() || shutdown.has_changed().is_err() {
            return Ok(());
        }
        let context = create_app_context.create_app_context(Some(format!("cron/{}", job.id)), None);
        // Dependency errors are logged by the usecase/adapter. Failed or timed-out
        // executions are retried at the next daily trigger, without cron-level logs.
        let _ = timeout(job.execution_timeout, job.handler.execute(&context)).await;
        // Recompute from now: runs never overlap and missed days are not replayed.
    }
}

async fn wait_for_shutdown(shutdown: &mut watch::Receiver<bool>) {
    loop {
        if *shutdown.borrow_and_update() {
            return;
        }
        if shutdown.changed().await.is_err() {
            return;
        }
    }
}

fn next_run(now: SystemTime, at: NaiveTime) -> Result<SystemTime, CronError> {
    let elapsed = now
        .duration_since(UNIX_EPOCH)
        .map_err(|_| CronError::Clock)?;
    let seconds = i64::try_from(elapsed.as_secs()).map_err(|_| CronError::Clock)?;
    let now =
        DateTime::<Utc>::from_timestamp(seconds, elapsed.subsec_nanos()).ok_or(CronError::Clock)?;
    let mut next = now.date_naive().and_time(at);
    if next <= now.naive_utc() {
        next = next
            .checked_add_signed(chrono::Duration::days(1))
            .ok_or(CronError::Clock)?;
    }
    let next = next.and_utc();
    UNIX_EPOCH
        .checked_add(Duration::new(
            u64::try_from(next.timestamp()).map_err(|_| CronError::Clock)?,
            next.timestamp_subsec_nanos(),
        ))
        .ok_or(CronError::Clock)
}

#[derive(Debug)]
pub enum CronError {
    Clock,
    Worker(tokio::task::JoinError),
}
impl fmt::Display for CronError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clock => formatter.write_str("cron clock is outside the supported range"),
            Self::Worker(_) => formatter.write_str("cron worker stopped unexpectedly"),
        }
    }
}
impl Error for CronError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Worker(source) => Some(source),
            Self::Clock => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CronError, next_run};
    use chrono::{DateTime, NaiveTime, Utc};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn at(value: &str) -> SystemTime {
        value.parse::<DateTime<Utc>>().unwrap().into()
    }

    #[test]
    fn daily_schedule_uses_next_utc_occurrence_including_year_rollover() {
        for (now, expected) in [
            ("2026-10-24T23:59:59Z", "2026-10-25T00:00:00Z"),
            ("2026-10-24T00:00:00Z", "2026-10-25T00:00:00Z"),
            ("2026-12-31T23:59:59Z", "2027-01-01T00:00:00Z"),
        ] {
            assert_eq!(next_run(at(now), NaiveTime::MIN).unwrap(), at(expected));
        }
        assert!(matches!(
            next_run(
                UNIX_EPOCH - std::time::Duration::from_secs(1),
                NaiveTime::MIN
            ),
            Err(CronError::Clock)
        ));
    }
}
