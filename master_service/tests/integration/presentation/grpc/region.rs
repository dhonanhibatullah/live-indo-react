use std::sync::Arc;

use live_indo_proto::live_indo_react::master::v1::{
    self as proto, geographic_region_service_client::GeographicRegionServiceClient,
    geographic_region_service_server::GeographicRegionServiceServer,
};
use master_service::{
    application::misc::AppContextService,
    infrastructure::utility::UuidAppContextFactory,
    presentation::grpc::{interceptor, server::geographic::region::RegionServer},
};
use prost::Message;
use tokio::{sync::oneshot, task::JoinHandle};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    Code, Request,
    transport::{Channel, Server},
};
use uuid::Uuid;

use crate::support::region::{Failure, Mode, RegionUsecase};

struct TestServer {
    client: GeographicRegionServiceClient<Channel>,
    shutdown: Option<oneshot::Sender<()>>,
    task: Option<JoinHandle<Result<(), tonic::transport::Error>>>,
}
impl TestServer {
    async fn start(usecase: Arc<RegionUsecase>) -> Self {
        let context = Arc::new(AppContextService::new(Arc::new(
            UuidAppContextFactory::new(),
        )));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (shutdown, stopped) = oneshot::channel();
        let task = tokio::spawn(async move {
            Server::builder()
                .layer(interceptor::layer(context))
                .add_service(GeographicRegionServiceServer::new(RegionServer::new(
                    usecase,
                )))
                .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                    let _ = stopped.await;
                })
                .await
        });
        let client = GeographicRegionServiceClient::connect(format!("http://{address}"))
            .await
            .unwrap();
        Self {
            client,
            shutdown: Some(shutdown),
            task: Some(task),
        }
    }
    async fn stop(mut self) {
        let _ = self.shutdown.take().unwrap().send(());
        self.task.take().unwrap().await.unwrap().unwrap();
    }
}
impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
fn request<T>(input: T) -> Request<T> {
    let mut request = Request::new(input);
    request
        .metadata_mut()
        .insert("x-actor", "region-test".parse().unwrap());
    request.metadata_mut().insert(
        "x-trace-id",
        "00000000-0000-0000-0000-00000000002a".parse().unwrap(),
    );
    request
}

#[tokio::test]
async fn grpc_ip_lookup_maps_presence_and_forwards_ip_and_context_through_generated_client() {
    let usecase = Arc::new(RegionUsecase::new());
    let mut server = TestServer::start(usecase.clone()).await;
    for ip in ["203.0.113.1", "2001:db8::1"] {
        for mode in [
            Mode::Full,
            Mode::EstimateOnly,
            Mode::NoCoordinates,
            Mode::Unknown,
        ] {
            usecase.set(mode);
            let response = server
                .client
                .get_region_by_ip(request(proto::GetRegionByIpRequest { ip: ip.into() }))
                .await
                .unwrap()
                .into_inner();
            if matches!(mode, Mode::Unknown) {
                assert!(response.geoip.is_none());
            } else {
                let geoip = response.geoip.unwrap();
                assert_eq!(geoip.country_code.as_deref(), Some("ID"));
                assert_eq!(geoip.subdivisions[0].code.as_deref(), Some("JK"));
                assert_eq!(geoip.accuracy_radius_km, Some(10));
                assert_eq!(geoip.time_zone.as_deref(), Some("Asia/Jakarta"));
                assert!(geoip.postal_code.is_none());
                if matches!(mode, Mode::NoCoordinates) {
                    assert!(geoip.coordinates.is_none());
                } else {
                    assert_eq!(geoip.coordinates.unwrap().longitude, 106.8);
                }
            }
            if matches!(mode, Mode::Full) {
                let region = response.region.unwrap();
                assert_eq!(region.province.unwrap().kemendagri_code, "31");
                assert_eq!(region.city.unwrap().kemendagri_code, "31.01");
            } else {
                assert!(response.region.is_none());
            }
            let calls = usecase.calls.lock().unwrap();
            let (context, observed) = calls.last().unwrap();
            assert_eq!(*observed, ip.parse::<std::net::IpAddr>().unwrap());
            assert_eq!(context.actor.as_deref(), Some("region-test"));
            assert_eq!(context.trace_id, Some(Uuid::from_u128(42)));
            assert!(context.execution_id.is_some());
        }
    }
    let existing = server
        .client
        .get_region_by_coord(request(proto::GetRegionByCoordRequest {
            longitude: 106.8,
            latitude: -6.2,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        existing.region.unwrap().province.unwrap().kemendagri_code,
        "31"
    );
    server.stop().await;
}

#[tokio::test]
async fn grpc_ip_lookup_rejects_malformed_input_and_missing_context_before_usecase() {
    let usecase = Arc::new(RegionUsecase::new());
    let mut server = TestServer::start(usecase.clone()).await;
    for ip in [
        "",
        "not-an-ip",
        "203.0.113.1:80",
        "203.0.113.1/24",
        "[2001:db8::1]",
        " 203.0.113.1",
    ] {
        let error = server
            .client
            .get_region_by_ip(request(proto::GetRegionByIpRequest { ip: ip.into() }))
            .await
            .unwrap_err();
        assert_eq!(error.code(), Code::InvalidArgument);
        assert_eq!(error.message(), "ip must be an IPv4 or IPv6 address");
        let detail = proto::Error::decode(error.details()).unwrap();
        assert_eq!(detail.code, proto::ErrorCode::BadArgs as i32);
    }
    let missing = server
        .client
        .get_region_by_ip(proto::GetRegionByIpRequest {
            ip: "203.0.113.1".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(missing.code(), Code::InvalidArgument);
    assert!(usecase.calls.lock().unwrap().is_empty());
    server.stop().await;
}

#[tokio::test]
async fn grpc_ip_lookup_preserves_classified_errors_without_exposing_provider_causes() {
    let usecase = Arc::new(RegionUsecase::new());
    let mut server = TestServer::start(usecase.clone()).await;
    for (failure, code) in [
        (Failure::Configuration, Code::Internal),
        (Failure::NotReady, Code::Unavailable),
        (Failure::Database, Code::Internal),
        (Failure::Download, Code::Internal),
        (Failure::Unavailable, Code::Unavailable),
        (Failure::Timeout, Code::DeadlineExceeded),
        (Failure::Operation, Code::Internal),
    ] {
        usecase.set(Mode::Error(failure));
        let error = server
            .client
            .get_region_by_ip(request(proto::GetRegionByIpRequest {
                ip: "203.0.113.1".into(),
            }))
            .await
            .unwrap_err();
        assert_eq!(error.code(), code);
        assert!(!error.message().contains("private-provider-secret"));
        assert!(!error.message().contains("203.0.113.1"));
        let detail = proto::Error::decode(error.details()).unwrap();
        assert_eq!(
            proto::ErrorCode::try_from(detail.code)
                .unwrap()
                .as_str_name(),
            format!("ERROR_CODE_{}", failure.error().code())
        );
        assert_eq!(detail.message, error.message());
    }
    server.stop().await;
}
