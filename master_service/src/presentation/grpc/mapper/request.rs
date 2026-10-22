use chrono::NaiveDate;
use live_indo_proto::live_indo_react::master::v1 as proto;
use tonic::Status;

use crate::domain::{
    models::entity::CityKind,
    usecases::{
        Page,
        management::{
            city::{CreateCity, UpdateCity},
            indonesia_boundary::UpdateIndonesiaBoundary,
            polling_method::UpdatePollingMethod,
            province::{CreateProvince, UpdateProvince},
            topic::{CreateTopic, UpdateTopic},
        },
        topic::{ListPollingMethods, ListTopics},
    },
};

use super::{
    audit::parse_date,
    error::bad_args,
    geometry::{optional_boundary, required_boundary},
};

pub(crate) fn create_topic(value: proto::CreateTopicRequest) -> CreateTopic {
    CreateTopic {
        slug: value.slug,
        name: value.name,
        description: value.description,
        polling_method_id: value.polling_method_id,
        selection_options: value.selection_options,
        is_active: value.is_active,
    }
}

pub(crate) fn update_topic(value: proto::UpdateTopicRequest) -> UpdateTopic {
    UpdateTopic {
        id: value.id,
        name: value.name,
        description: value.description,
        polling_method_id: value.polling_method_id,
        selection_options: value.selection_options.map(|options| options.values),
        is_active: value.is_active,
    }
}

pub(crate) fn update_polling_method(
    value: proto::UpdatePollingMethodRequest,
) -> UpdatePollingMethod {
    UpdatePollingMethod {
        id: value.id,
        name: value.name,
        description: value.description,
        is_active: value.is_active,
    }
}

pub(crate) fn create_province(
    value: proto::CreateProvinceRequest,
) -> Result<CreateProvince, Status> {
    Ok(CreateProvince {
        kemendagri_code: value.kemendagri_code,
        name: value.name,
        boundary: required_boundary(value.boundary)?,
        boundary_source: value.boundary_source,
        boundary_reference_date: value.boundary_reference_date.map(parse_date).transpose()?,
        is_active: value.is_active,
    })
}

pub(crate) fn update_province(
    value: proto::UpdateProvinceRequest,
) -> Result<UpdateProvince, Status> {
    Ok(UpdateProvince {
        id: value.id,
        name: value.name,
        boundary: optional_boundary(value.boundary)?,
        boundary_source: value.boundary_source,
        boundary_reference_date: province_date_change(value.boundary_reference_date_change)?,
        is_active: value.is_active,
    })
}

pub(crate) fn create_city(value: proto::CreateCityRequest) -> Result<CreateCity, Status> {
    Ok(CreateCity {
        province_id: value.province_id,
        kemendagri_code: value.kemendagri_code,
        kind: parse_city_kind(value.kind)?,
        name: value.name,
        boundary: required_boundary(value.boundary)?,
        boundary_source: value.boundary_source,
        boundary_reference_date: value.boundary_reference_date.map(parse_date).transpose()?,
        is_active: value.is_active,
    })
}

pub(crate) fn update_city(value: proto::UpdateCityRequest) -> Result<UpdateCity, Status> {
    Ok(UpdateCity {
        id: value.id,
        kind: value.kind.map(parse_city_kind).transpose()?,
        name: value.name,
        boundary: optional_boundary(value.boundary)?,
        boundary_source: value.boundary_source,
        boundary_reference_date: city_date_change(value.boundary_reference_date_change)?,
        is_active: value.is_active,
    })
}

pub(crate) fn update_indonesia_boundary(
    value: proto::UpdateIndonesiaBoundaryRequest,
) -> Result<UpdateIndonesiaBoundary, Status> {
    Ok(UpdateIndonesiaBoundary {
        boundary: optional_boundary(value.boundary)?,
        boundary_source: value.boundary_source,
        boundary_reference_date: boundary_date_change(value.boundary_reference_date_change)?,
    })
}

pub(crate) fn list_polling_methods(
    value: proto::ListAvailablePollingMethodsRequest,
) -> Result<ListPollingMethods, Status> {
    Ok(ListPollingMethods {
        page: page(value.page)?,
    })
}

pub(crate) fn list_topics(value: proto::ListActiveTopicsRequest) -> Result<ListTopics, Status> {
    Ok(ListTopics {
        polling_method_id: value.polling_method_id,
        name_contains: value.name_contains,
        page: page(value.page)?,
    })
}

fn page(value: Option<proto::PageRequest>) -> Result<Page, Status> {
    let value = value.ok_or_else(|| bad_args("page is required"))?;
    Ok(Page {
        limit: value.limit,
        offset: value.offset,
    })
}

fn parse_city_kind(value: i32) -> Result<CityKind, Status> {
    match proto::CityKind::try_from(value).map_err(|_| bad_args("unknown city kind"))? {
        proto::CityKind::City => Ok(CityKind::City),
        proto::CityKind::Regency => Ok(CityKind::Regency),
        proto::CityKind::Unspecified => Err(bad_args("city kind is required")),
    }
}

fn province_date_change(
    value: Option<proto::update_province_request::BoundaryReferenceDateChange>,
) -> Result<Option<Option<NaiveDate>>, Status> {
    match value {
        None => Ok(None),
        Some(
            proto::update_province_request::BoundaryReferenceDateChange::BoundaryReferenceDate(
                value,
            ),
        ) => Ok(Some(Some(parse_date(value)?))),
        Some(
            proto::update_province_request::BoundaryReferenceDateChange::ClearBoundaryReferenceDate(
                true,
            ),
        ) => Ok(Some(None)),
        Some(
            proto::update_province_request::BoundaryReferenceDateChange::ClearBoundaryReferenceDate(
                false,
            ),
        ) => Err(bad_args(
            "clear_boundary_reference_date must be true when set",
        )),
    }
}

fn city_date_change(
    value: Option<proto::update_city_request::BoundaryReferenceDateChange>,
) -> Result<Option<Option<NaiveDate>>, Status> {
    match value {
        None => Ok(None),
        Some(proto::update_city_request::BoundaryReferenceDateChange::BoundaryReferenceDate(
            value,
        )) => Ok(Some(Some(parse_date(value)?))),
        Some(
            proto::update_city_request::BoundaryReferenceDateChange::ClearBoundaryReferenceDate(
                true,
            ),
        ) => Ok(Some(None)),
        Some(
            proto::update_city_request::BoundaryReferenceDateChange::ClearBoundaryReferenceDate(
                false,
            ),
        ) => Err(bad_args(
            "clear_boundary_reference_date must be true when set",
        )),
    }
}

fn boundary_date_change(
    value: Option<proto::update_indonesia_boundary_request::BoundaryReferenceDateChange>,
) -> Result<Option<Option<NaiveDate>>, Status> {
    match value {
        None => Ok(None),
        Some(
            proto::update_indonesia_boundary_request::BoundaryReferenceDateChange::BoundaryReferenceDate(
                value,
            ),
        ) => Ok(Some(Some(parse_date(value)?))),
        Some(
            proto::update_indonesia_boundary_request::BoundaryReferenceDateChange::ClearBoundaryReferenceDate(
                true,
            ),
        ) => Ok(Some(None)),
        Some(
            proto::update_indonesia_boundary_request::BoundaryReferenceDateChange::ClearBoundaryReferenceDate(
                false,
            ),
        ) => Err(bad_args("clear_boundary_reference_date must be true when set")),
    }
}
