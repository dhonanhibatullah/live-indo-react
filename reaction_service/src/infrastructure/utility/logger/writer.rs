use std::{
    io::{self, Write},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use serde_json::json;

use crate::domain::{
    contracts::utility::logger::Logger,
    models::logger::{Format, Level, Meta},
};

use super::normalize::normalize_meta;

/// An injected, synchronous structured writer with no global registration.
///
/// Calls serialize a complete line, then lock the writer through write and flush.
/// Output can block; use an appropriate writer or offload calls from async tasks.
/// Emission failures are silently ignored; potentially partial writes are not retried.
pub struct WriterLogger<W> {
    writer: Mutex<W>,
    format: Format,
    level: Level,
}

impl<W: Write + Send> WriterLogger<W> {
    pub fn new(writer: W, format: Format, level: Level) -> Self {
        Self {
            writer: Mutex::new(writer),
            format,
            level,
        }
    }
}

impl<W: Write + Send> WriterLogger<W> {
    fn write_record(
        &self,
        level: Level,
        tag: &str,
        message: &str,
        meta: Option<&Meta>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let meta = normalize_meta(meta);
        let mut line = match self.format {
            Format::Json => serde_json::to_vec(&json!({
                "time_unix_ms": time,
                "level": level.as_str(),
                "msg": message,
                "tag": tag,
                "meta": meta,
            }))?,
            Format::Plain => {
                // JSON string escaping keeps user-supplied text on one physical line.
                let tag = serde_json::to_string(tag)?;
                let message = serde_json::to_string(message)?;
                format!("time_unix_ms={time} level={level} msg={message} tag={tag} meta={meta}")
                    .into_bytes()
            }
        };
        line.push(b'\n');

        let mut writer = self
            .writer
            .lock()
            .map_err(|_| io::Error::other("logger writer lock poisoned"))?;
        writer.write_all(&line)?;
        Ok(writer.flush()?)
    }
}

impl<W: Write + Send> Logger for WriterLogger<W> {
    fn log(&self, level: Level, tag: &str, message: &str, meta: Option<&Meta>) {
        if !self.level.allows(level) {
            return;
        }

        let _ = self.write_record(level, tag, message, meta);
    }
}
