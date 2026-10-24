use std::{sync::Arc, time::Duration};

use chrono::NaiveTime;

use crate::domain::{models::AppContext, usecases::UsecaseFuture};

pub(crate) trait JobHandler: Send + Sync {
    fn execute<'a>(&'a self, context: &'a AppContext) -> UsecaseFuture<'a, ()>;
}

/// A daily UTC trigger. One worker per job prevents overlap and catch-up queues.
pub struct Job {
    pub id: &'static str,
    pub time_utc: NaiveTime,
    pub execution_timeout: Duration,
    pub(crate) handler: Arc<dyn JobHandler>,
}
