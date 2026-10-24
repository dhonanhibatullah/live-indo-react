use std::{
    collections::VecDeque,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use flate2::{Compression, write::GzEncoder};
use master_service::{
    domain::{
        contracts::{service::GeoIpService, utility::logger::Logger},
        models::{
            AppContext,
            app::GeoIpConf,
            error::GeoIpError,
            logger::{Format, Level},
        },
    },
    infrastructure::{service::geoip::HttpsGeoIp, utility::logger::WriterLogger},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    task::JoinHandle,
};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{self, pki_types::PrivatePkcs8KeyDer},
};

use crate::support::logger::CapturingLogger;

const CITY: &[u8] = include_bytes!("../../../fixtures/geoip/GeoIP2-City-Test.mmdb");
const COUNTRY: &[u8] = include_bytes!("../../../fixtures/geoip/GeoIP2-Country-Test.mmdb");
const RELEASE: &str = "Wed, 21 Oct 2026 12:00:00 GMT";
const NEXT_RELEASE: &str = "Fri, 23 Oct 2026 12:00:00 GMT";

#[derive(Clone)]
struct Reply {
    status: u16,
    declared_length: Option<usize>,
    modified: Option<&'static str>,
    body: Vec<u8>,
    body_gate: Option<Arc<tokio::sync::Notify>>,
    body_started: Option<Arc<tokio::sync::Notify>>,
}
impl Reply {
    fn ok(modified: &'static str, body: Vec<u8>) -> Self {
        Self {
            status: 200,
            declared_length: None,
            modified: Some(modified),
            body,
            body_gate: None,
            body_started: None,
        }
    }
}

struct Server {
    client: reqwest::Client,
    url: reqwest::Url,
    replies: Arc<Mutex<VecDeque<Reply>>>,
    requests: Arc<Mutex<Vec<String>>>,
    task: JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl Server {
    async fn start(replies: Vec<Reply>) -> Self {
        let rcgen::CertifiedKey { cert, signing_key } =
            rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let root = reqwest::Certificate::from_der(cert.der()).unwrap();
        let configuration = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                vec![cert.der().clone()],
                PrivatePkcs8KeyDer::from(signing_key.serialize_der()).into(),
            )
            .unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(configuration));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let client = reqwest::Client::builder()
            .https_only(true)
            .no_proxy()
            .add_root_certificate(root)
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let url = format!(
            "https://localhost:{}/download?suffix=tar.gz",
            address.port()
        )
        .parse()
        .unwrap();
        let replies = Arc::new(Mutex::new(VecDeque::<Reply>::from(replies)));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let queue = replies.clone();
        let captured = requests.clone();
        let task = tokio::spawn(async move {
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let mut stream = acceptor.accept(socket).await.unwrap();
                let mut request = Vec::new();
                loop {
                    let byte = stream.read_u8().await.unwrap();
                    request.push(byte);
                    if request.ends_with(b"\r\n\r\n") {
                        break;
                    }
                    assert!(request.len() < 8192);
                }
                let request = String::from_utf8(request).unwrap();
                let head = request.starts_with("HEAD ");
                captured.lock().unwrap().push(request);
                let response = queue
                    .lock()
                    .unwrap()
                    .pop_front()
                    .expect("unexpected request");
                let modified = response
                    .modified
                    .map(|date| format!("Last-Modified: {date}\r\n"))
                    .unwrap_or_default();
                let headers = format!(
                    "HTTP/1.1 {} Test\r\n{modified}Content-Length: {}\r\nConnection: close\r\n\r\n",
                    response.status,
                    response.declared_length.unwrap_or(response.body.len())
                );
                stream.write_all(headers.as_bytes()).await.unwrap();
                if !head {
                    if let Some(started) = response.body_started {
                        started.notify_one();
                    }
                    if let Some(gate) = response.body_gate {
                        gate.notified().await;
                    }
                    stream.write_all(&response.body).await.unwrap();
                }
                stream.shutdown().await.unwrap();
            }
        });
        Self {
            client,
            url,
            replies,
            requests,
            task,
        }
    }

    async fn adapter(&self, path: &std::path::Path) -> HttpsGeoIp {
        self.adapter_with_logger(path, logger()).await
    }

    async fn adapter_with_logger(
        &self,
        path: &std::path::Path,
        logger: Arc<dyn Logger>,
    ) -> HttpsGeoIp {
        HttpsGeoIp::with_client(conf(), path, logger, self.client.clone(), self.url.clone())
            .await
            .unwrap()
    }
    fn exhausted(&self) {
        assert!(self.replies.lock().unwrap().is_empty());
    }
}

fn conf() -> GeoIpConf {
    GeoIpConf {
        account_id: 123,
        license_key: "fixture-key".into(),
        edition_ids: vec!["GeoLite2-City".into()],
    }
}
fn logger() -> Arc<dyn Logger> {
    Arc::new(WriterLogger::new(
        std::io::sink(),
        Format::Json,
        Level::None,
    ))
}
fn archive(content: &[u8]) -> Vec<u8> {
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut archive = tar::Builder::new(encoder);
    let mut header = tar::Header::new_gnu();
    header.set_size(content.len() as u64);
    header.set_mode(0o600);
    header.set_cksum();
    archive
        .append_data(
            &mut header,
            "GeoLite2-City_20261023/GeoLite2-City.mmdb",
            content,
        )
        .unwrap();
    archive.into_inner().unwrap().finish().unwrap()
}

#[tokio::test]
async fn downloads_queries_reuses_release_and_reopens_without_network() {
    let server = Server::start(vec![
        Reply::ok(RELEASE, archive(CITY)),
        Reply::ok(RELEASE, Vec::new()),
        Reply::ok(RELEASE, Vec::new()),
    ])
    .await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("city.mmdb");
    let adapter = server.adapter(&path).await;
    let context = AppContext::default();
    assert!(!adapter.is_ready(&context).await.unwrap());
    assert!(adapter.check_for_update(&context).await.unwrap());
    assert!(matches!(
        adapter
            .get_location(&context, "89.160.20.128".parse().unwrap())
            .await,
        Err(GeoIpError::NotReady)
    ));
    adapter.update(&context).await.unwrap();
    assert!(adapter.is_ready(&context).await.unwrap());
    assert_eq!(std::fs::read(&path).unwrap(), CITY);
    assert!(!adapter.check_for_update(&context).await.unwrap());
    adapter.update(&context).await.unwrap();
    assert_location(&adapter, &context).await;
    let reopened = server.adapter(&path).await;
    assert_location(&reopened, &context).await;
    let requests = server.requests.lock().unwrap();
    assert_eq!(requests.len(), 3);
    assert!(requests[0].starts_with("GET "));
    assert!(
        requests[1..]
            .iter()
            .all(|request| request.starts_with("HEAD "))
    );
    // Synthetic 123:fixture-key credentials only; never actual account secrets.
    assert!(requests.iter().all(|request| {
        request
            .to_lowercase()
            .contains("authorization: basic mtizomzpehr1cmuta2v5")
    }));
    server.exhausted();
}

async fn assert_location(adapter: &HttpsGeoIp, context: &AppContext) {
    let ip: IpAddr = "89.160.20.128".parse().unwrap();
    let location = adapter.get_location(context, ip).await.unwrap().unwrap();
    assert_eq!(location.country_code.as_deref(), Some("SE"));
    assert_eq!(location.city_name.as_deref(), Some("Linköping"));
    let coordinate = location.coordinates.unwrap();
    assert!((coordinate.latitude - 58.4167).abs() < 0.001);
    assert!((coordinate.longitude - 15.6167).abs() < 0.001);
    assert!(
        adapter
            .get_location(context, "::1".parse().unwrap())
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        adapter
            .get_location(context, "2001:218::".parse().unwrap())
            .await
            .unwrap()
            .is_some()
    );
}

#[tokio::test]
async fn replaces_changed_release_and_serializes_simultaneous_updates() {
    let server = Server::start(vec![
        Reply::ok(RELEASE, archive(CITY)),
        Reply::ok(NEXT_RELEASE, Vec::new()),
        Reply::ok(NEXT_RELEASE, Vec::new()),
        Reply::ok(NEXT_RELEASE, archive(CITY)),
        Reply::ok(NEXT_RELEASE, Vec::new()),
    ])
    .await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("city.mmdb");
    let adapter = server.adapter(&path).await;
    let context = AppContext::default();
    adapter.update(&context).await.unwrap();
    assert!(adapter.check_for_update(&context).await.unwrap());
    let (first, second) = tokio::join!(adapter.update(&context), adapter.update(&context));
    first.unwrap();
    second.unwrap();
    assert_location(&adapter, &context).await;
    assert_eq!(
        std::fs::metadata(path).unwrap().modified().unwrap(),
        httpdate::parse_http_date(NEXT_RELEASE).unwrap()
    );
    assert_eq!(
        server
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|request| request.starts_with("GET "))
            .count(),
        2
    );
    server.exhausted();
}

#[tokio::test]
async fn corrupt_downloads_and_wrong_editions_preserve_current_database() {
    for invalid in [
        b"not gzip".to_vec(),
        archive(COUNTRY),
        archive(b"not an mmdb"),
        {
            let mut damaged = archive(CITY);
            let last = damaged.len() - 1;
            damaged[last] ^= 1;
            damaged
        },
    ] {
        let server = Server::start(vec![
            Reply::ok(RELEASE, archive(CITY)),
            Reply::ok(NEXT_RELEASE, Vec::new()),
            Reply::ok(NEXT_RELEASE, invalid),
        ])
        .await;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("city.mmdb");
        let adapter = server.adapter(&path).await;
        let context = AppContext::default();
        adapter.update(&context).await.unwrap();
        assert!(adapter.update(&context).await.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), CITY);
        assert_location(&adapter, &context).await;
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        server.exhausted();
    }
}

#[tokio::test]
async fn remote_failures_are_not_mistaken_for_current_and_corrupt_local_file_recovers() {
    let server = Server::start(vec![
        Reply::ok(RELEASE, archive(CITY)),
        Reply {
            status: 429,
            declared_length: None,
            modified: None,
            body: Vec::new(),
            body_gate: None,
            body_started: None,
        },
        Reply {
            status: 200,
            declared_length: None,
            modified: None,
            body: Vec::new(),
            body_gate: None,
            body_started: None,
        },
    ])
    .await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("city.mmdb");
    std::fs::write(&path, b"corrupt local database").unwrap();
    let adapter = server.adapter(&path).await;
    let context = AppContext::default();
    assert!(!adapter.is_ready(&context).await.unwrap());
    adapter.update(&context).await.unwrap();
    assert!(matches!(
        adapter.check_for_update(&context).await,
        Err(GeoIpError::Unavailable)
    ));
    assert!(matches!(
        adapter.check_for_update(&context).await,
        Err(GeoIpError::Unavailable)
    ));
    assert_location(&adapter, &context).await;
    server.exhausted();
}

#[tokio::test]
async fn validates_city_configuration_without_network() {
    let directory = tempfile::tempdir().unwrap();
    let mut config = conf();
    config.edition_ids = vec!["GeoLite2-Country".into()];
    assert!(matches!(
        HttpsGeoIp::new(config, directory.path().join("city.mmdb"), logger()).await,
        Err(GeoIpError::InvalidConfiguration)
    ));
}

#[tokio::test]
async fn cancelled_download_preserves_file_and_keeps_lookups_available() {
    let started = Arc::new(tokio::sync::Notify::new());
    let gate = Arc::new(tokio::sync::Notify::new());
    let mut stalled = Reply::ok(NEXT_RELEASE, archive(CITY));
    stalled.body_started = Some(started.clone());
    stalled.body_gate = Some(gate);
    let server = Server::start(vec![
        Reply::ok(RELEASE, archive(CITY)),
        Reply::ok(NEXT_RELEASE, Vec::new()),
        stalled,
    ])
    .await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("city.mmdb");
    let adapter = Arc::new(server.adapter(&path).await);
    let context = AppContext::default();
    adapter.update(&context).await.unwrap();
    let worker = adapter.clone();
    let update = tokio::spawn(async move { worker.update(&AppContext::default()).await });
    tokio::time::timeout(Duration::from_secs(5), started.notified())
        .await
        .unwrap();
    assert_location(&adapter, &context).await;
    update.abort();
    assert!(update.await.unwrap_err().is_cancelled());
    assert_eq!(std::fs::read(&path).unwrap(), CITY);
    assert_eq!(
        std::fs::metadata(&path).unwrap().modified().unwrap(),
        httpdate::parse_http_date(RELEASE).unwrap()
    );
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    server.exhausted();
}

#[tokio::test]
async fn oversized_download_is_rejected_before_reading_or_replacing_data() {
    let mut oversized = Reply::ok(NEXT_RELEASE, Vec::new());
    oversized.declared_length = Some(256 * 1024 * 1024 + 1);
    let server = Server::start(vec![
        Reply::ok(RELEASE, archive(CITY)),
        Reply::ok(NEXT_RELEASE, Vec::new()),
        oversized,
    ])
    .await;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("city.mmdb");
    let adapter = server.adapter(&path).await;
    let context = AppContext::default();
    adapter.update(&context).await.unwrap();
    assert!(matches!(
        adapter.update(&context).await,
        Err(GeoIpError::InvalidDownload { .. })
    ));
    assert_eq!(std::fs::read(&path).unwrap(), CITY);
    assert_location(&adapter, &context).await;
    server.exhausted();
}

#[tokio::test]
async fn update_logs_installation_skip_and_failure_as_distinct_outcomes() {
    let server = Server::start(vec![
        Reply::ok(RELEASE, archive(CITY)),
        Reply::ok(RELEASE, Vec::new()),
        Reply::ok(NEXT_RELEASE, Vec::new()),
        Reply::ok(NEXT_RELEASE, b"invalid archive".to_vec()),
    ])
    .await;
    let directory = tempfile::tempdir().unwrap();
    let logger = Arc::new(CapturingLogger::default());
    let adapter = server
        .adapter_with_logger(&directory.path().join("city.mmdb"), logger.clone())
        .await;
    let context = AppContext::default();
    adapter.update(&context).await.unwrap();
    adapter.update(&context).await.unwrap();
    assert!(adapter.update(&context).await.is_err());
    let records = logger.records();
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].level, Level::Info);
    assert_eq!(records[0].message, "GeoIP database updated");
    assert_eq!(records[1].level, Level::Info);
    assert_eq!(
        records[1].message,
        "GeoIP update skipped: database is current"
    );
    assert_eq!(records[2].level, Level::Error);
    assert_eq!(records[2].message, "GeoIP operation failed");
    assert_eq!(
        records[2].meta.as_ref().unwrap()["error_code"],
        "GEOIP_DOWNLOAD_INVALID".into()
    );
    assert!(records.iter().all(|record| record.tag == "geoip/update"));
    server.exhausted();
}
