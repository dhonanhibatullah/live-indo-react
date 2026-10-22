use chrono::{DateTime, Datelike, NaiveDate, Utc};
use live_indo_proto::live_indo_react::master::v1 as proto;
use prost_types::Timestamp;
use tonic::Status;

use super::error::bad_args;

pub(crate) fn timestamp(value: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: value.timestamp(),
        nanos: value.timestamp_subsec_nanos() as i32,
    }
}

pub(crate) fn optional_timestamp(value: Option<DateTime<Utc>>) -> Option<Timestamp> {
    value.map(timestamp)
}

pub(crate) fn audit(
    created_at: DateTime<Utc>,
    updated_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
) -> proto::AuditFields {
    proto::AuditFields {
        created_at: Some(timestamp(created_at)),
        updated_at: optional_timestamp(updated_at),
        deleted_at: optional_timestamp(deleted_at),
    }
}

pub(crate) fn date(value: NaiveDate) -> proto::Date {
    proto::Date {
        year: value.year(),
        month: value.month() as i32,
        day: value.day() as i32,
    }
}

pub(crate) fn parse_date(value: proto::Date) -> Result<NaiveDate, Status> {
    NaiveDate::from_ymd_opt(value.year, value.month as u32, value.day as u32)
        .ok_or_else(|| bad_args("boundary_reference_date is not a valid date"))
}
