use std::sync::Mutex;

use master_service::domain::{
    contracts::utility::Logger,
    models::{
        AppContext,
        logger::{Level, Meta},
    },
};

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
        self.records.lock().unwrap().clone()
    }
}

impl Logger for CapturingLogger {
    fn log(
        &self,
        _context: &AppContext,
        level: Level,
        tag: &str,
        message: &str,
        meta: Option<&Meta>,
    ) {
        self.records.lock().unwrap().push(LogRecord {
            level,
            tag: tag.into(),
            message: message.into(),
            meta: meta.cloned(),
        });
    }
}
