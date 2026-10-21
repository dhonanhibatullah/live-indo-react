use crate::domain::models::logger::{Level, Meta};

/// Best-effort structured logging without a process-global logger dependency.
///
/// Metadata is borrowed and never mutated. None is emitted as an empty object.
/// Filtered calls do not touch the output. Emission failures are silently ignored.
pub trait Logger: Send + Sync {
    fn log(&self, level: Level, tag: &str, message: &str, meta: Option<&Meta>);

    fn error(&self, tag: &str, message: &str, meta: Option<&Meta>) {
        self.log(Level::Error, tag, message, meta);
    }

    fn warn(&self, tag: &str, message: &str, meta: Option<&Meta>) {
        self.log(Level::Warn, tag, message, meta);
    }

    fn info(&self, tag: &str, message: &str, meta: Option<&Meta>) {
        self.log(Level::Info, tag, message, meta);
    }

    fn debug(&self, tag: &str, message: &str, meta: Option<&Meta>) {
        self.log(Level::Debug, tag, message, meta);
    }
}
