use std::{future::Future, io, sync::Arc};

use super::{
    application, driver, infrastructure, presentation,
    runner::{self, LaunchError},
    startup,
};
use crate::domain::{contracts::utility::Logger, models::AppEnv};

pub(super) struct Wiring {
    pub startup: startup::Startup,
    pub presentation: presentation::Presentation,
}

pub(super) async fn run(
    env: &AppEnv,
    logger: Arc<dyn Logger>,
    shutdown: impl Future<Output = Result<(), io::Error>>,
) -> Result<(), LaunchError> {
    let drivers = driver::build(env, logger)
        .await
        .map_err(|error| LaunchError::Database(Box::new(error)))?;
    runner::run(env, &drivers, wire(env, &drivers), shutdown).await
}

async fn wire(env: &AppEnv, drivers: &driver::Drivers) -> Result<Wiring, LaunchError> {
    let infrastructure = infrastructure::build(drivers, env)
        .await
        .map_err(LaunchError::GeoIp)?;
    let geoip = infrastructure.geoip.clone();
    let application = application::build(infrastructure, drivers.logger.clone());
    let presentation = presentation::build(&application);
    let startup = startup::build(&application, geoip);
    Ok(Wiring {
        startup,
        presentation,
    })
}
