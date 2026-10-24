use std::{sync::Arc, time::Duration};

use chrono::NaiveTime;

use crate::{
    domain::usecases::maintenance::RefreshGeoIp,
    presentation::cron::{handler::maintenance::geoip::GeoIpHandler, job::Job},
};

pub fn jobs(refresh_geoip: Arc<dyn RefreshGeoIp>) -> Vec<Job> {
    vec![Job {
        id: "maintenance.geoip.refresh",
        time_utc: NaiveTime::MIN,
        execution_timeout: Duration::from_secs(600),
        handler: Arc::new(GeoIpHandler {
            refresh: refresh_geoip,
        }),
    }]
}
