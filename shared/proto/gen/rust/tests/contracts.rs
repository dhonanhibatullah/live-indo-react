use live_indo_proto::live_indo_react::master::v1::*;
use prost::Message;

#[test]
fn optional_empty_selection_options_preserve_presence() {
    let request = UpdateTopicRequest {
        id: 1,
        name: None,
        description: None,
        polling_method_id: None,
        selection_options: Some(StringList { values: Vec::new() }),
        is_active: None,
    };
    let encoded = request.encode_to_vec();
    let decoded = UpdateTopicRequest::decode(encoded.as_slice()).expect("decode update topic");

    assert!(decoded.selection_options.is_some());
    assert!(decoded.selection_options.unwrap().values.is_empty());
}

#[test]
fn boundary_reference_date_change_preserves_clear_and_set_presence() {
    let clear = UpdateProvinceRequest {
        id: 1,
        name: None,
        boundary: None,
        boundary_source: None,
        boundary_reference_date_change: Some(
            update_province_request::BoundaryReferenceDateChange::ClearBoundaryReferenceDate(true),
        ),
        is_active: None,
    };
    let decoded =
        UpdateProvinceRequest::decode(clear.encode_to_vec().as_slice()).expect("decode clear date");
    assert!(matches!(
        decoded.boundary_reference_date_change,
        Some(
            update_province_request::BoundaryReferenceDateChange::ClearBoundaryReferenceDate(true)
        )
    ));

    let set = UpdateProvinceRequest {
        boundary_reference_date_change: Some(
            update_province_request::BoundaryReferenceDateChange::BoundaryReferenceDate(Date {
                year: 2026,
                month: 10,
                day: 22,
            }),
        ),
        ..clear
    };
    let decoded =
        UpdateProvinceRequest::decode(set.encode_to_vec().as_slice()).expect("decode set date");
    assert!(matches!(
        decoded.boundary_reference_date_change,
        Some(
            update_province_request::BoundaryReferenceDateChange::BoundaryReferenceDate(Date {
                year: 2026,
                month: 10,
                day: 22
            })
        )
    ));
}

#[test]
fn error_codes_follow_the_application_vocabulary() {
    assert_eq!(ErrorCode::Unspecified as i32, 0);
    assert_eq!(ErrorCode::BadArgs.as_str_name(), "ERROR_CODE_BAD_ARGS");
    assert_eq!(
        ErrorCode::TopicPollingMethodNotFound.as_str_name(),
        "ERROR_CODE_TOPIC_POLLING_METHOD_NOT_FOUND"
    );
    assert_eq!(
        ErrorCode::IndonesiaBoundaryNotFound.as_str_name(),
        "ERROR_CODE_INDONESIA_BOUNDARY_NOT_FOUND"
    );
}

#[test]
fn generated_service_types_are_available() {
    fn geographic(
        _: Option<
            geographic_province_service_client::GeographicProvinceServiceClient<
                tonic::transport::Channel,
            >,
        >,
    ) {
    }
    fn management(_: Option<management_topic_service_server::ManagementTopicServiceServer<()>>) {}
    fn topic(
        _: Option<
            topic_catalog_service_client::TopicCatalogServiceClient<tonic::transport::Channel>,
        >,
    ) {
    }

    geographic(None);
    management(None);
    topic(None);
}
