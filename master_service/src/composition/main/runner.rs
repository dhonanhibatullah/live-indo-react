use std::{error::Error, fmt, future::Future, io, process::ExitCode, sync::Arc};

use pgdt::sqlx::PgPool;
use tokio::{
    net::TcpListener,
    sync::{mpsc, watch},
    time::timeout,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use super::{
    driver,
    launcher::{self, Wiring},
    presentation, startup,
};
use crate::{
    config,
    domain::{
        contracts::utility::Logger,
        models::{AppContext, AppEnv, AppInfo, logger::Meta},
    },
    presentation::{cron::CronError, grpc::interceptor},
};

pub async fn launch() -> ExitCode {
    let env = match config::env::load() {
        Ok(env) => env,
        Err(error) => {
            eprintln!("[ERROR] load configuration: {error}");
            return ExitCode::FAILURE;
        }
    };
    let app_info = config::app::load();
    let logger = driver::logger(&env);
    let context = AppContext::default();

    match run_entry(env, app_info, logger.clone(), shutdown_signal()).await {
        Ok(()) => {
            logger.info(&context, "composition/launch", "service stopped", None);
            ExitCode::SUCCESS
        }
        Err(error) => {
            logger.error(
                &context,
                "composition/launch",
                "service failed",
                Some(&Meta::from([("reason".into(), error.to_string().into())])),
            );
            ExitCode::FAILURE
        }
    }
}

/// Run the service with an injected shutdown future. Intended for composition tests.
pub async fn launch_with(
    env: AppEnv,
    shutdown: impl Future<Output = Result<(), io::Error>> + Send,
) -> Result<(), LaunchError> {
    let logger = driver::logger(&env);
    run_entry(env, config::app::load(), logger, shutdown).await
}

async fn run_entry(
    env: AppEnv,
    app_info: AppInfo,
    logger: Arc<dyn Logger>,
    shutdown: impl Future<Output = Result<(), io::Error>> + Send,
) -> Result<(), LaunchError> {
    let context = AppContext::default();
    let meta = Meta::from([
        ("name".into(), app_info.name.into()),
        ("version".into(), app_info.version.into()),
        ("http_addr".into(), env.http_addr.to_string().into()),
        ("grpc_addr".into(), env.grpc_addr.to_string().into()),
    ]);
    logger.info(
        &context,
        "composition/launch",
        "service starting",
        Some(&meta),
    );
    launcher::run(&env, logger, shutdown).await
}

/// Supervise wiring, startup and execution while retaining the driver for cleanup.
pub(super) async fn run(
    env: &AppEnv,
    drivers: &driver::Drivers,
    wiring: impl Future<Output = Result<Wiring, LaunchError>>,
    shutdown: impl Future<Output = Result<(), io::Error>>,
) -> Result<(), LaunchError> {
    let result = async {
        let wiring = wiring.await?;
        startup::run(&wiring.startup).await?;
        run_servers(env, &drivers.logger, wiring.presentation, shutdown).await
    }
    .await;
    close_pool(drivers.database.pool(), env.shutdown_timeout, result).await
}

async fn close_pool(
    pool: &PgPool,
    deadline: std::time::Duration,
    result: Result<(), LaunchError>,
) -> Result<(), LaunchError> {
    let closed = timeout(deadline, pool.close())
        .await
        .map_err(|_| LaunchError::ShutdownTimeout);
    // Cleanup must run on failure too, without replacing the original failure.
    result.and(closed)
}

async fn run_servers(
    env: &AppEnv,
    logger: &Arc<dyn Logger>,
    presentations: presentation::Presentation,
    shutdown: impl Future<Output = Result<(), io::Error>>,
) -> Result<(), LaunchError> {
    let presentation::Presentation {
        http: http_router,
        grpc: grpc_routes,
        cron,
        create_app_context,
    } = presentations;
    let http_listener =
        TcpListener::bind(env.http_addr)
            .await
            .map_err(|source| LaunchError::Bind {
                transport: Transport::Http,
                source,
            })?;
    let grpc_listener =
        TcpListener::bind(env.grpc_addr)
            .await
            .map_err(|source| LaunchError::Bind {
                transport: Transport::Grpc,
                source,
            })?;

    let http_addr = http_listener
        .local_addr()
        .map_err(|source| LaunchError::Bind {
            transport: Transport::Http,
            source,
        })?;
    let grpc_addr = grpc_listener
        .local_addr()
        .map_err(|source| LaunchError::Bind {
            transport: Transport::Grpc,
            source,
        })?;
    let listening = Meta::from([
        ("http_addr".into(), http_addr.to_string().into()),
        ("grpc_addr".into(), grpc_addr.to_string().into()),
    ]);
    logger.info(
        &AppContext::default(),
        "composition/listeners",
        "transport listeners bound",
        Some(&listening),
    );

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (exit_tx, mut exit_rx) = mpsc::channel(3);

    let http_exit = exit_tx.clone();
    let http_shutdown = shutdown_rx.clone();
    let mut http_task = tokio::spawn(async move {
        let result = axum::serve(http_listener, http_router)
            .with_graceful_shutdown(wait_for_shutdown(http_shutdown))
            .await;
        let _ = http_exit.send(Transport::Http).await;
        result
    });

    let grpc_shutdown = shutdown_rx.clone();
    let grpc_exit = exit_tx.clone();
    let mut grpc_task = tokio::spawn(async move {
        let result = Server::builder()
            .layer(interceptor::layer(create_app_context))
            .add_routes(grpc_routes)
            .serve_with_incoming_shutdown(
                TcpListenerStream::new(grpc_listener),
                wait_for_shutdown(grpc_shutdown),
            )
            .await;
        let _ = grpc_exit.send(Transport::Grpc).await;
        result
    });

    let mut cron_task = tokio::spawn(async move {
        let result = cron.run(shutdown_rx).await;
        let _ = exit_tx.send(Transport::Cron).await;
        result
    });

    tokio::pin!(shutdown);
    let mut signal_error = None;
    let mut early_exit = None;
    tokio::select! {
        result = &mut shutdown => {
            if let Err(error) = result {
                signal_error = Some(LaunchError::Signal(error));
            }
        }
        Some(transport) = exit_rx.recv() => {
            early_exit = Some(transport);
        }
    }

    let _ = shutdown_tx.send(true);
    let drained = timeout(env.shutdown_timeout, async {
        tokio::join!(&mut http_task, &mut grpc_task, &mut cron_task)
    })
    .await;

    let (http_result, grpc_result, cron_result) = match drained {
        Ok(results) => results,
        Err(_) => {
            http_task.abort();
            grpc_task.abort();
            cron_task.abort();
            let _ = tokio::join!(http_task, grpc_task, cron_task);
            return Err(LaunchError::ShutdownTimeout);
        }
    };

    if let Some(error) = signal_error {
        return Err(error);
    }
    if let Some(error) = http_failure(http_result) {
        return Err(error);
    }
    if let Some(error) = grpc_failure(grpc_result) {
        return Err(error);
    }
    if let Some(error) = cron_failure(cron_result) {
        return Err(error);
    }
    if let Some(transport) = early_exit {
        return Err(LaunchError::ServerStopped { transport });
    }

    Ok(())
}

async fn wait_for_shutdown(mut shutdown: watch::Receiver<bool>) {
    if *shutdown.borrow() {
        return;
    }
    let _ = shutdown.changed().await;
}

fn http_failure(
    result: Result<Result<(), io::Error>, tokio::task::JoinError>,
) -> Option<LaunchError> {
    match result {
        Ok(Ok(())) => None,
        Ok(Err(source)) => Some(LaunchError::Serve {
            transport: Transport::Http,
            source: Box::new(source),
        }),
        Err(source) => Some(LaunchError::Serve {
            transport: Transport::Http,
            source: Box::new(source),
        }),
    }
}

fn grpc_failure(
    result: Result<Result<(), tonic::transport::Error>, tokio::task::JoinError>,
) -> Option<LaunchError> {
    match result {
        Ok(Ok(())) => None,
        Ok(Err(source)) => Some(LaunchError::Serve {
            transport: Transport::Grpc,
            source: Box::new(source),
        }),
        Err(source) => Some(LaunchError::Serve {
            transport: Transport::Grpc,
            source: Box::new(source),
        }),
    }
}

fn cron_failure(
    result: Result<Result<(), CronError>, tokio::task::JoinError>,
) -> Option<LaunchError> {
    let source: Box<dyn Error + Send + Sync> = match result {
        Ok(Ok(())) => return None,
        Ok(Err(source)) => Box::new(source),
        Err(source) => Box::new(source),
    };
    Some(LaunchError::Serve {
        transport: Transport::Cron,
        source,
    })
}

async fn shutdown_signal() -> Result<(), io::Error> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = signal(SignalKind::terminate())?;
        tokio::select! {
            result = tokio::signal::ctrl_c() => result,
            _ = terminate.recv() => Ok(()),
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Transport {
    Http,
    Grpc,
    Cron,
}

impl Transport {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Grpc => "grpc",
            Self::Cron => "cron",
        }
    }
}

#[derive(Debug)]
pub enum LaunchError {
    Signal(io::Error),
    Database(Box<dyn Error + Send + Sync>),
    GeoIp(Box<dyn Error + Send + Sync>),
    Bind {
        transport: Transport,
        source: io::Error,
    },
    Serve {
        transport: Transport,
        source: Box<dyn Error + Send + Sync>,
    },
    ServerStopped {
        transport: Transport,
    },
    ShutdownTimeout,
}

impl fmt::Display for LaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signal(_) => formatter.write_str("could not install shutdown signal handling"),
            Self::GeoIp(_) => {
                formatter.write_str("could not initialize GeoIP configuration or database")
            }
            Self::Database(_) => formatter.write_str("could not initialize the database pool"),
            Self::Bind { transport, .. } => {
                write!(
                    formatter,
                    "could not bind the {} listener",
                    transport.as_str()
                )
            }
            Self::Serve { transport, .. } => {
                write!(formatter, "{} server failed", transport.as_str())
            }
            Self::ServerStopped { transport } => {
                write!(
                    formatter,
                    "{} server stopped unexpectedly",
                    transport.as_str()
                )
            }
            Self::ShutdownTimeout => formatter.write_str("service shutdown exceeded its deadline"),
        }
    }
}

impl Error for LaunchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Signal(source) => Some(source),
            Self::Database(source) | Self::GeoIp(source) => Some(source.as_ref()),
            Self::Bind { source, .. } => Some(source),
            Self::Serve { source, .. } => Some(source.as_ref()),
            Self::ServerStopped { .. } | Self::ShutdownTimeout => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        future::pending,
        net::SocketAddr,
        sync::{
            Mutex,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::Duration,
    };

    use axum::Router;
    use pgdt::{
        Pgdt,
        sqlx::postgres::{PgConnectOptions, PgPoolOptions},
    };
    use tonic::service::Routes;

    use super::*;
    use crate::{
        application::misc::AppContextService,
        domain::{
            contracts::service::{GeoIpFuture, GeoIpService},
            models::{
                GeoIpError,
                entity::GeoIp,
                logger::{Format, Level},
            },
            usecases::{
                UsecaseFuture, maintenance::RefreshGeoIp, misc::app_context::CreateAppContext,
            },
        },
        infrastructure::utility::UuidAppContextFactory,
        presentation::cron::Runner as CronRunner,
    };

    #[derive(Default)]
    struct Logs(Mutex<Vec<String>>);
    impl Logger for Logs {
        fn log(&self, _: &AppContext, _: Level, tag: &str, _: &str, _: Option<&Meta>) {
            self.0.lock().unwrap().push(tag.to_owned());
        }
    }

    #[derive(Default)]
    struct GeoIpDouble {
        ready: AtomicBool,
        contexts: Mutex<Vec<AppContext>>,
    }
    impl GeoIpService for GeoIpDouble {
        fn is_ready<'a>(&'a self, context: &'a AppContext) -> GeoIpFuture<'a, bool> {
            self.contexts.lock().unwrap().push(context.clone());
            Box::pin(async { Ok(self.ready.load(Ordering::SeqCst)) })
        }
        fn check_for_update<'a>(&'a self, _: &'a AppContext) -> GeoIpFuture<'a, bool> {
            panic!("startup must use the wired refresh usecase")
        }
        fn update<'a>(&'a self, _: &'a AppContext) -> GeoIpFuture<'a, ()> {
            panic!("startup must use the wired refresh usecase")
        }
        fn get_location<'a>(
            &'a self,
            _: &'a AppContext,
            _: std::net::IpAddr,
        ) -> GeoIpFuture<'a, Option<GeoIp>> {
            panic!("startup must not query locations")
        }
    }

    enum Outcome {
        Ready,
        Failure,
        Unready,
        Pending,
    }
    struct RefreshDouble {
        outcome: Outcome,
        geoip: Arc<GeoIpDouble>,
        presentations_built: Arc<AtomicBool>,
        calls: AtomicUsize,
        contexts: Mutex<Vec<AppContext>>,
    }
    impl RefreshGeoIp for RefreshDouble {
        fn refresh_geoip<'a>(&'a self, context: &'a AppContext) -> UsecaseFuture<'a, ()> {
            Box::pin(async move {
                assert!(self.presentations_built.load(Ordering::SeqCst));
                self.calls.fetch_add(1, Ordering::SeqCst);
                self.contexts.lock().unwrap().push(context.clone());
                match self.outcome {
                    Outcome::Ready => {
                        self.geoip.ready.store(true, Ordering::SeqCst);
                        Ok(())
                    }
                    Outcome::Failure => Err(GeoIpError::Unavailable.into()),
                    Outcome::Unready => Ok(()),
                    Outcome::Pending => pending().await,
                }
            })
        }
    }

    fn env() -> AppEnv {
        AppEnv {
            logger_format: Format::Plain,
            logger_level: Level::None,
            postgres_host: "127.0.0.1".into(),
            postgres_port: 5432,
            postgres_username: "fixture".into(),
            postgres_password: String::new(),
            postgres_database: "fixture".into(),
            postgres_ssl_mode: "disable".into(),
            postgres_max_connections: 1,
            postgres_connect_timeout: Duration::from_millis(100),
            cache_max_capacity: 10,
            cache_individual_ttl: Duration::from_secs(1),
            cache_list_ttl: Duration::from_secs(1),
            geoip_conf_path: "unused-test-GeoIP.conf".into(),
            geoip_database_path: "unused-test-city.mmdb".into(),
            http_addr: "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
            grpc_addr: "127.0.0.1:0".parse().unwrap(),
            shutdown_timeout: Duration::from_secs(1),
        }
    }

    fn drivers() -> (driver::Drivers, Arc<Logs>) {
        // No external database is contacted: no connection is acquired from this pool.
        let pool = PgPoolOptions::new().connect_lazy_with(PgConnectOptions::new());
        let logger = Arc::new(Logs::default());
        (
            driver::Drivers {
                database: Pgdt::new(pool),
                logger: logger.clone(),
            },
            logger,
        )
    }

    fn wiring(outcome: Outcome) -> (Wiring, Arc<RefreshDouble>) {
        let presentations_built = Arc::new(AtomicBool::new(false));
        let geoip = Arc::new(GeoIpDouble::default());
        let refresh = Arc::new(RefreshDouble {
            outcome,
            geoip: geoip.clone(),
            presentations_built: presentations_built.clone(),
            calls: AtomicUsize::new(0),
            contexts: Mutex::new(Vec::new()),
        });
        let context: Arc<dyn CreateAppContext> = Arc::new(AppContextService::new(Arc::new(
            UuidAppContextFactory::new(),
        )));
        let presentation = presentation::Presentation {
            http: Router::new(),
            grpc: Routes::default(),
            cron: CronRunner::new(context.clone(), Vec::new()),
            create_app_context: context.clone(),
        };
        presentations_built.store(true, Ordering::SeqCst);
        (
            Wiring {
                startup: startup::Startup {
                    refresh_geoip: refresh.clone(),
                    create_app_context: context,
                    geoip,
                },
                presentation,
            },
            refresh,
        )
    }

    #[tokio::test]
    async fn wired_startup_runs_once_before_listeners_and_all_presentations_drain() {
        let env = env();
        let (drivers, logs) = drivers();
        let (wiring, refresh) = wiring(Outcome::Ready);
        run(&env, &drivers, async { Ok(wiring) }, async { Ok(()) })
            .await
            .unwrap();
        assert_eq!(refresh.calls.load(Ordering::SeqCst), 1);
        let contexts = refresh.contexts.lock().unwrap();
        assert_eq!(contexts[0].actor.as_deref(), Some("startup/geoip.refresh"));
        assert!(contexts[0].execution_id.is_some());
        assert!(contexts[0].transaction.is_none());
        assert_eq!(*refresh.geoip.contexts.lock().unwrap(), *contexts);
        assert_eq!(*logs.0.lock().unwrap(), ["composition/listeners"]);
        assert!(drivers.database.pool().is_closed());
    }

    #[tokio::test]
    async fn infrastructure_wiring_failure_closes_the_previously_created_pool() {
        let directory = tempfile::tempdir().unwrap();
        let mut env = env();
        env.geoip_conf_path = directory.path().join("missing-GeoIP.conf");
        let (drivers, logs) = drivers();
        let wiring = async {
            super::super::infrastructure::build(&drivers, &env)
                .await
                .map_err(LaunchError::GeoIp)?;
            panic!("missing configuration should prevent infrastructure construction")
        };
        let error = run(&env, &drivers, wiring, pending()).await.unwrap_err();
        assert!(matches!(error, LaunchError::GeoIp(_)));
        assert!(
            error
                .source()
                .unwrap()
                .is::<crate::config::geoip_conf::LoadError>()
        );
        assert!(drivers.database.pool().is_closed());
        assert!(logs.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn refresh_errors_and_failed_readiness_prevent_binding_and_close_pool() {
        for outcome in [Outcome::Failure, Outcome::Unready] {
            let mut env = env();
            // Occupied port proves binding was not attempted before the failing startup.
            let occupied = TcpListener::bind("127.0.0.1:0").await.unwrap();
            env.http_addr = occupied.local_addr().unwrap();
            let (drivers, logs) = drivers();
            let (wiring, refresh) = wiring(outcome);
            let error = run(&env, &drivers, async { Ok(wiring) }, pending())
                .await
                .unwrap_err();
            assert!(matches!(error, LaunchError::GeoIp(_)));
            let source = error
                .source()
                .unwrap()
                .downcast_ref::<crate::domain::models::UsecaseError>()
                .unwrap();
            assert!(matches!(
                source.code(),
                "GEOIP_UNAVAILABLE" | "GEOIP_NOT_READY"
            ));
            assert_eq!(refresh.calls.load(Ordering::SeqCst), 1);
            assert!(drivers.database.pool().is_closed());
            assert!(logs.0.lock().unwrap().is_empty());
        }
    }

    #[tokio::test(start_paused = true)]
    async fn startup_deadline_prevents_binding_and_closes_pool() {
        let env = env();
        let (drivers, logs) = drivers();
        let (wiring, refresh) = wiring(Outcome::Pending);
        let began = tokio::time::Instant::now();
        let error = run(&env, &drivers, async { Ok(wiring) }, pending())
            .await
            .unwrap_err();
        assert!(matches!(error, LaunchError::GeoIp(_)));
        assert!(error.source().unwrap().is::<tokio::time::error::Elapsed>());
        assert_eq!(began.elapsed(), Duration::from_secs(600));
        assert_eq!(refresh.calls.load(Ordering::SeqCst), 1);
        assert!(drivers.database.pool().is_closed());
        assert!(logs.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn bind_failure_after_startup_also_closes_pool() {
        let mut env = env();
        let occupied = TcpListener::bind("127.0.0.1:0").await.unwrap();
        env.http_addr = occupied.local_addr().unwrap();
        let (drivers, _) = drivers();
        let (wiring, refresh) = wiring(Outcome::Ready);
        let error = run(&env, &drivers, async { Ok(wiring) }, pending())
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            LaunchError::Bind {
                transport: Transport::Http,
                ..
            }
        ));
        assert_eq!(refresh.calls.load(Ordering::SeqCst), 1);
        assert!(drivers.database.pool().is_closed());
    }
}
