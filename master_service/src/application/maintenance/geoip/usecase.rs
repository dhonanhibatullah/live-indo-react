use std::sync::Arc;

use crate::domain::{
    contracts::{service::GeoIpService, utility::Logger},
    models::AppContext,
    usecases::{UsecaseFuture, maintenance::RefreshGeoIp},
};

pub struct GeoIpMaintenance {
    geoip: Arc<dyn GeoIpService>,
    logger: Arc<dyn Logger>,
}

impl GeoIpMaintenance {
    pub fn new(geoip: Arc<dyn GeoIpService>, logger: Arc<dyn Logger>) -> Self {
        Self { geoip, logger }
    }
}

impl RefreshGeoIp for GeoIpMaintenance {
    fn refresh_geoip<'a>(&'a self, context: &'a AppContext) -> UsecaseFuture<'a, ()> {
        Box::pin(async move {
            if !self.geoip.is_ready(context).await? {
                self.geoip.update(context).await?;
                return Ok(());
            }

            if self.geoip.check_for_update(context).await? {
                self.geoip.update(context).await?;
            } else {
                self.logger.info(
                    context,
                    "maintenance/geoip/refresh",
                    "GeoIP refresh skipped: database is current",
                    None,
                );
            }

            Ok(())
        })
    }
}
