use std::{error::Error, fmt, future::Future, io, process::ExitCode, sync::Arc};

use axum::Router;
use pgdt::Pgdt;
use tokio::{
    net::TcpListener,
    sync::{mpsc, watch},
    time::timeout,
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{service::Routes, transport::Server};

use crate::composition::main::{application, driver, infrastructure, presentation};

use crate::{
    config,
    domain::{
        contracts::utility::Logger,
        models::{AppContext, AppEnv, AppInfo, logger::Meta},
        usecases::misc::app_context::CreateAppContext,
    },
    presentation::grpc::interceptor,
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

    match run(env, app_info, logger.clone(), shutdown_signal()).await {
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
    run(env, config::app::load(), logger, shutdown).await
}

async fn run(
    env: AppEnv,
    app_info: AppInfo,
    logger: Arc<dyn Logger>,
    shutdown: impl Future<Output = Result<(), io::Error>> + Send,
) -> Result<(), LaunchError> {
    let context = AppContext::default();
    let startup = Meta::from([
        ("name".into(), app_info.name.into()),
        ("version".into(), app_info.version.into()),
        ("http_addr".into(), env.http_addr.to_string().into()),
        ("grpc_addr".into(), env.grpc_addr.to_string().into()),
    ]);
    logger.info(
        &context,
        "composition/launch",
        "service starting",
        Some(&startup),
    );

    let pool = driver::connect(&env)
        .await
        .map_err(|error| LaunchError::Database(Box::new(error)))?;
    let database = Pgdt::new(pool.clone());
    let infrastructure = infrastructure::build(database, logger.clone(), &env);
    let application = application::build(infrastructure, logger.clone());
    let http = presentation::http(&application);
    let grpc = presentation::grpc(&application);
    let create_app_context = application.create_app_context.clone();

    let result = run_servers(&env, &logger, http, grpc, create_app_context, shutdown).await;

    match timeout(env.shutdown_timeout, pool.close()).await {
        Ok(()) => result,
        Err(_) => Err(LaunchError::ShutdownTimeout),
    }
}

async fn run_servers(
    env: &AppEnv,
    logger: &Arc<dyn Logger>,
    http_router: Router,
    grpc_routes: Routes,
    create_app_context: Arc<dyn CreateAppContext>,
    shutdown: impl Future<Output = Result<(), io::Error>>,
) -> Result<(), LaunchError> {
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
    let (exit_tx, mut exit_rx) = mpsc::channel(2);

    let http_exit = exit_tx.clone();
    let http_shutdown = shutdown_rx.clone();
    let mut http_task = tokio::spawn(async move {
        let result = axum::serve(http_listener, http_router)
            .with_graceful_shutdown(wait_for_shutdown(http_shutdown))
            .await;
        let _ = http_exit.send(Transport::Http).await;
        result
    });

    let grpc_shutdown = shutdown_rx;
    let mut grpc_task = tokio::spawn(async move {
        let result = Server::builder()
            .layer(interceptor::layer(create_app_context))
            .add_routes(grpc_routes)
            .serve_with_incoming_shutdown(
                TcpListenerStream::new(grpc_listener),
                wait_for_shutdown(grpc_shutdown),
            )
            .await;
        let _ = exit_tx.send(Transport::Grpc).await;
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
        tokio::join!(&mut http_task, &mut grpc_task)
    })
    .await;

    let (http_result, grpc_result) = match drained {
        Ok(results) => results,
        Err(_) => {
            http_task.abort();
            grpc_task.abort();
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
}

impl Transport {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Grpc => "grpc",
        }
    }
}

#[derive(Debug)]
pub enum LaunchError {
    Signal(io::Error),
    Database(Box<dyn Error + Send + Sync>),
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
            Self::Database(source) => Some(source.as_ref()),
            Self::Bind { source, .. } => Some(source),
            Self::Serve { source, .. } => Some(source.as_ref()),
            Self::ServerStopped { .. } | Self::ShutdownTimeout => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, time::Duration};

    use axum::Router;
    use tokio::sync::oneshot;
    use tonic::service::Routes;

    use super::run_servers;
    use crate::{
        application::misc::AppContextService,
        domain::models::{
            AppEnv,
            logger::{Format, Level},
        },
        infrastructure::utility::{UuidAppContextFactory, logger::WriterLogger},
    };

    fn env() -> AppEnv {
        AppEnv {
            logger_format: Format::Plain,
            logger_level: Level::None,
            postgres_host: "127.0.0.1".to_owned(),
            postgres_port: 5432,
            postgres_username: "user".to_owned(),
            postgres_password: String::new(),
            postgres_database: "master_database".to_owned(),
            postgres_ssl_mode: "prefer".to_owned(),
            postgres_max_connections: 1,
            postgres_connect_timeout: Duration::from_millis(100),
            cache_max_capacity: 10,
            cache_individual_ttl: Duration::from_secs(1),
            cache_list_ttl: Duration::from_secs(1),
            http_addr: "127.0.0.1:0".parse::<SocketAddr>().expect("http addr"),
            grpc_addr: "127.0.0.1:0".parse::<SocketAddr>().expect("grpc addr"),
            shutdown_timeout: Duration::from_secs(1),
        }
    }

    #[tokio::test]
    async fn listeners_shut_down_without_process_signals() {
        let env = env();
        let logger: std::sync::Arc<dyn crate::domain::contracts::utility::Logger> =
            std::sync::Arc::new(WriterLogger::new(
                Vec::new(),
                env.logger_format,
                env.logger_level,
            ));
        let context = std::sync::Arc::new(AppContextService::new(std::sync::Arc::new(
            UuidAppContextFactory::new(),
        )));
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        shutdown_tx.send(()).expect("shutdown receiver");

        let result = run_servers(
            &env,
            &logger,
            Router::new(),
            Routes::default(),
            context,
            async move { shutdown_rx.await.map_err(std::io::Error::other) },
        )
        .await;

        assert!(result.is_ok(), "{result:?}");
    }
}
