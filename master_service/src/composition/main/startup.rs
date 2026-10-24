use std::{sync::Arc, time::Duration};

use tokio::time::timeout;

use super::{application::Application, runner::LaunchError};
use crate::domain::{
    contracts::service::GeoIpService,
    models::{GeoIpError, UsecaseError},
    usecases::{maintenance::RefreshGeoIp, misc::app_context::CreateAppContext},
};

pub(super) struct Startup {
    pub refresh_geoip: Arc<dyn RefreshGeoIp>,
    pub create_app_context: Arc<dyn CreateAppContext>,
    pub geoip: Arc<dyn GeoIpService>,
}

pub(super) fn build(application: &Application, geoip: Arc<dyn GeoIpService>) -> Startup {
    Startup {
        refresh_geoip: application.refresh_geoip.clone(),
        create_app_context: application.create_app_context.clone(),
        geoip,
    }
}

/// Called only after every layer is wired, before binding or starting presentations.
pub(super) async fn run(startup: &Startup) -> Result<(), LaunchError> {
    let context = startup
        .create_app_context
        .create_app_context(Some("startup/geoip.refresh".into()), None);
    let initialize = async {
        startup.refresh_geoip.refresh_geoip(&context).await?;
        if !startup.geoip.is_ready(&context).await? {
            return Err(GeoIpError::NotReady.into());
        }
        Ok::<(), UsecaseError>(())
    };
    timeout(Duration::from_secs(600), initialize)
        .await
        .map_err(|source| LaunchError::GeoIp(Box::new(source)))?
        .map_err(|source| LaunchError::GeoIp(Box::new(source)))
}
