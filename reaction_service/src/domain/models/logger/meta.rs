use std::{collections::BTreeMap, error::Error};

/// Owned structured metadata, independent of any output library.
pub type Meta = BTreeMap<String, MetaValue>;

#[derive(Clone, Debug, PartialEq)]
pub enum MetaValue {
    Null,
    Bool(bool),
    Integer(i64),
    Unsigned(u64),
    Float(f64),
    String(String),
    Array(Vec<MetaValue>),
    Object(Meta),
}

impl MetaValue {
    /// Capture diagnostic text without retaining or serializing an error object.
    /// Callers must ensure this text contains no sensitive data.
    pub fn from_error(error: &(dyn Error + 'static)) -> Self {
        Self::String(error.to_string())
    }
}

impl From<&str> for MetaValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for MetaValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<bool> for MetaValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for MetaValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<u64> for MetaValue {
    fn from(value: u64) -> Self {
        Self::Unsigned(value)
    }
}

impl From<f64> for MetaValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
