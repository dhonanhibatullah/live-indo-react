use chrono::NaiveDate;

use crate::presentation::http::dto::response::error::ApiError;

pub(crate) fn parse_date(value: Option<String>) -> Result<Option<NaiveDate>, ApiError> {
    value
        .map(|value| {
            NaiveDate::parse_from_str(&value, "%Y-%m-%d")
                .map_err(|_| ApiError::bad_args("boundary_reference_date must be YYYY-MM-DD"))
        })
        .transpose()
}

pub(crate) fn date_change(
    value: Option<String>,
    clear: bool,
) -> Result<Option<Option<NaiveDate>>, ApiError> {
    if clear {
        if value.is_some() {
            return Err(ApiError::bad_args(
                "boundary_reference_date and clear_boundary_reference_date cannot both be set",
            ));
        }
        return Ok(Some(None));
    }

    Ok(parse_date(value)?.map(Some))
}
