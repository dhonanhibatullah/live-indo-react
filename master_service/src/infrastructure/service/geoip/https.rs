use std::{
    net::IpAddr,
    path::PathBuf,
    sync::Arc,
    time::{Duration, SystemTime},
};

use reqwest::{Client, Method, Response, header::LAST_MODIFIED};
use tokio::sync::{Mutex, RwLock, Semaphore};

use crate::domain::{
    contracts::{
        service::{GeoIpFuture, GeoIpService},
        utility::logger::Logger,
    },
    models::{AppContext, app::GeoIpConf, entity::GeoIp, error::GeoIpError, logger::Meta},
};

use super::{
    archive,
    reader::{self, Database, failure},
};

const DOWNLOAD_URL: &str =
    "https://download.maxmind.com/geoip/databases/GeoLite2-City/download?suffix=tar.gz";
const MAX_DOWNLOAD_BYTES: usize = 256 * 1024 * 1024;

/// MaxMind HTTPS downloads and an in-memory MMDB reader. One instance owns one
/// database path; coordinate shared-volume writers outside this adapter.
/// Local data remains available during downloads. No scheduler or retries are installed.
pub struct HttpsGeoIp {
    configuration: GeoIpConf,
    database_path: PathBuf,
    client: Client,
    download_url: reqwest::Url,
    logger: Arc<dyn Logger>,
    state: Arc<RwLock<Option<Arc<Database>>>>,
    update_lock: Arc<Mutex<()>>,
    lookup_slots: Arc<Semaphore>,
}

impl HttpsGeoIp {
    pub async fn new(
        configuration: GeoIpConf,
        database_path: impl Into<PathBuf>,
        logger: Arc<dyn Logger>,
    ) -> Result<Self, GeoIpError> {
        let client = Client::builder()
            .https_only(true)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(http_error)?;
        Self::with_client(
            configuration,
            database_path,
            logger,
            client,
            reqwest::Url::parse(DOWNLOAD_URL).map_err(failure)?,
        )
        .await
    }

    /// Inject a client for custom trust roots/proxy settings. It must enforce
    /// HTTPS, bounded timeouts/redirects, and normal cross-origin credential stripping.
    /// The endpoint is trusted configuration: it receives Basic-auth credentials.
    /// Production new() uses MaxMind; endpoint injection enables isolated HTTPS tests.
    pub async fn with_client(
        configuration: GeoIpConf,
        database_path: impl Into<PathBuf>,
        logger: Arc<dyn Logger>,
        client: Client,
        download_url: reqwest::Url,
    ) -> Result<Self, GeoIpError> {
        let database_path = database_path.into();
        if configuration.account_id == 0
            || configuration.license_key.trim().is_empty()
            || configuration.license_key.chars().any(char::is_whitespace)
            || !configuration
                .edition_ids
                .iter()
                .any(|edition| edition == "GeoLite2-City")
            || database_path.file_name().is_none()
            || download_url.scheme() != "https"
            || download_url.host_str().is_none()
            || !download_url.username().is_empty()
            || download_url.password().is_some()
        {
            return Err(GeoIpError::InvalidConfiguration);
        }
        let path = database_path.clone();
        let loaded = tokio::task::spawn_blocking(move || reader::load(&path))
            .await
            .map_err(failure)?;
        let database = match loaded {
            Ok(database) => database,
            Err(GeoIpError::InvalidDatabase { .. }) => {
                logger.warn(
                    &AppContext::default(),
                    "geoip/load",
                    "local GeoIP database needs replacement",
                    None,
                );
                None
            }
            Err(error) => return Err(error),
        };
        Ok(Self {
            configuration,
            database_path,
            client,
            download_url,
            logger,
            state: Arc::new(RwLock::new(database.map(Arc::new))),
            update_lock: Arc::new(Mutex::new(())),
            lookup_slots: Arc::new(Semaphore::new(64)),
        })
    }

    async fn request(&self, method: Method) -> Result<Response, GeoIpError> {
        let response = self
            .client
            .request(method, self.download_url.clone())
            .basic_auth(
                self.configuration.account_id.to_string(),
                Some(&self.configuration.license_key),
            )
            .send()
            .await
            .map_err(http_error)?;
        if !response.status().is_success() {
            return Err(GeoIpError::Unavailable);
        }
        Ok(response)
    }

    async fn needs_update(&self) -> Result<bool, GeoIpError> {
        let current = self.state.read().await.clone();
        let Some(current) = current else {
            return Ok(true);
        };
        let response = self.request(Method::HEAD).await?;
        Ok(last_modified(&response)? != current.modified)
    }

    async fn update_database(&self) -> Result<(), GeoIpError> {
        let permit = self.update_lock.clone().lock_owned().await;
        if !self.needs_update().await? {
            return Ok(());
        }
        let mut response = self.request(Method::GET).await?;
        let modified = last_modified(&response)?;
        if response
            .content_length()
            .is_some_and(|length| length > MAX_DOWNLOAD_BYTES as u64)
        {
            return Err(download_limit());
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(http_error)? {
            if chunk.len() > MAX_DOWNLOAD_BYTES.saturating_sub(bytes.len()) {
                return Err(download_limit());
            }
            bytes.extend_from_slice(&chunk);
        }
        let path = self.database_path.clone();
        let state = self.state.clone();
        tokio::task::spawn_blocking(move || {
            archive::install(bytes, modified, &path, &state, permit)
        })
        .await
        .map_err(failure)?
    }

    fn report<T>(
        &self,
        context: &AppContext,
        tag: &str,
        result: Result<T, GeoIpError>,
    ) -> Result<T, GeoIpError> {
        if let Err(error) = &result {
            let meta = Meta::from([("error_code".into(), error.code().into())]);
            self.logger
                .error(context, tag, "GeoIP operation failed", Some(&meta));
        }
        result
    }
}

impl GeoIpService for HttpsGeoIp {
    fn is_ready<'a>(&'a self, _context: &'a AppContext) -> GeoIpFuture<'a, bool> {
        Box::pin(async { Ok(self.state.read().await.is_some()) })
    }

    fn check_for_update<'a>(&'a self, context: &'a AppContext) -> GeoIpFuture<'a, bool> {
        Box::pin(async move {
            self.report(context, "geoip/check_for_update", self.needs_update().await)
        })
    }

    fn update<'a>(&'a self, context: &'a AppContext) -> GeoIpFuture<'a, ()> {
        Box::pin(async move {
            self.report(context, "geoip/update", self.update_database().await)?;
            self.logger
                .info(context, "geoip/update", "GeoIP database is current", None);
            Ok(())
        })
    }

    fn get_location<'a>(
        &'a self,
        context: &'a AppContext,
        ip: IpAddr,
    ) -> GeoIpFuture<'a, Option<GeoIp>> {
        Box::pin(async move {
            let database = self.state.read().await.clone();
            let result = match database {
                Some(database) => {
                    let permit = self
                        .lookup_slots
                        .clone()
                        .acquire_owned()
                        .await
                        .map_err(failure)?;
                    tokio::task::spawn_blocking(move || {
                        let _permit = permit;
                        reader::lookup(&database, ip)
                    })
                    .await
                    .map_err(failure)
                    .and_then(|result| result)
                }
                None => Err(GeoIpError::NotReady),
            };
            self.report(context, "geoip/get_location", result)
        })
    }
}

fn http_error(source: reqwest::Error) -> GeoIpError {
    let timeout = source.is_timeout();
    // Redirect URLs can contain download signatures; never retain them in errors.
    let source = Box::new(source.without_url());
    if timeout {
        GeoIpError::Timeout { source }
    } else {
        GeoIpError::Failure { source }
    }
}

fn last_modified(response: &Response) -> Result<SystemTime, GeoIpError> {
    response
        .headers()
        .get(LAST_MODIFIED)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| httpdate::parse_http_date(value).ok())
        .ok_or(GeoIpError::Unavailable)
}

fn download_limit() -> GeoIpError {
    GeoIpError::InvalidDownload {
        source: Box::new(std::io::Error::other("download exceeds size limit")),
    }
}
