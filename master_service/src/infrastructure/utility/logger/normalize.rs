use serde_json::{Map, Number, Value};

use crate::domain::models::logger::{Meta, MetaValue};

pub(super) fn normalize_meta(meta: Option<&Meta>) -> Value {
    Value::Object(meta.map(normalize_object).unwrap_or_default())
}

fn normalize_object(meta: &Meta) -> Map<String, Value> {
    meta.iter()
        .map(|(key, value)| (key.clone(), normalize_value(value)))
        .collect()
}

fn normalize_value(value: &MetaValue) -> Value {
    match value {
        MetaValue::Null => Value::Null,
        MetaValue::Bool(value) => Value::Bool(*value),
        MetaValue::Integer(value) => Value::Number((*value).into()),
        MetaValue::Unsigned(value) => Value::Number((*value).into()),
        // JSON cannot represent NaN or infinity. Match JSON's null convention.
        MetaValue::Float(value) => Number::from_f64(*value).map_or(Value::Null, Value::Number),
        MetaValue::String(value) => Value::String(value.clone()),
        MetaValue::Array(values) => Value::Array(values.iter().map(normalize_value).collect()),
        MetaValue::Object(values) => Value::Object(normalize_object(values)),
    }
}
