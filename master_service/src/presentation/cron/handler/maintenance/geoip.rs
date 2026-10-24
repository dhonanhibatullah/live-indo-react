use std::sync::Arc;

use crate::{
    domain::{
        models::AppContext,
        usecases::{UsecaseFuture, maintenance::RefreshGeoIp},
    },
    presentation::cron::job::JobHandler,
};

pub(crate) struct GeoIpHandler {
    pub refresh: Arc<dyn RefreshGeoIp>,
}

impl JobHandler for GeoIpHandler {
    fn execute<'a>(&'a self, context: &'a AppContext) -> UsecaseFuture<'a, ()> {
        self.refresh.refresh_geoip(context)
    }
}
