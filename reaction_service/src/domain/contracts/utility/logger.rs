use crate::domain::models::{
    AppContext,
    logger::{Level, Meta},
};

/// Best-effort structured logging without a process-global logger dependency.
///
/// Metadata is borrowed and never mutated. None is emitted as an empty object.
/// Filtered calls do not touch the output. Emission failures are silently ignored.
pub trait Logger: Send + Sync {
    fn log(
        &self,
        context: &AppContext,
        level: Level,
        tag: &str,
        message: &str,
        meta: Option<&Meta>,
    );

    fn error(&self, context: &AppContext, tag: &str, message: &str, meta: Option<&Meta>) {
        self.log(context, Level::Error, tag, message, meta);
    }

    fn warn(&self, context: &AppContext, tag: &str, message: &str, meta: Option<&Meta>) {
        self.log(context, Level::Warn, tag, message, meta);
    }

    fn info(&self, context: &AppContext, tag: &str, message: &str, meta: Option<&Meta>) {
        self.log(context, Level::Info, tag, message, meta);
    }

    fn debug(&self, context: &AppContext, tag: &str, message: &str, meta: Option<&Meta>) {
        self.log(context, Level::Debug, tag, message, meta);
    }
}
