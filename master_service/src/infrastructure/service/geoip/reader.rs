use std::{
    fs,
    io::{self, Read},
    net::IpAddr,
    path::Path,
    time::SystemTime,
};

use maxminddb::{Reader, geoip2};

use crate::domain::models::{
    entity::{GeoIp, GeoIpCoordinates, GeoIpSubdivision},
    error::GeoIpError,
};

pub(super) const MAX_DATABASE_BYTES: u64 = 512 * 1024 * 1024;

pub(super) struct Database {
    pub reader: Reader<Vec<u8>>,
    pub modified: SystemTime,
}

pub(super) fn failure(source: impl std::error::Error + Send + Sync + 'static) -> GeoIpError {
    GeoIpError::Failure {
        source: Box::new(source),
    }
}

pub(super) fn validate(bytes: Vec<u8>) -> Result<Reader<Vec<u8>>, GeoIpError> {
    let invalid = |source| GeoIpError::InvalidDatabase {
        source: Box::new(source),
    };
    let reader = Reader::from_source(bytes).map_err(invalid)?;
    if !matches!(
        reader.metadata().database_type.as_str(),
        "GeoLite2-City" | "GeoIP2-City"
    ) {
        return Err(GeoIpError::InvalidDatabase {
            source: Box::new(io::Error::other("expected a City database")),
        });
    }
    reader.verify().map_err(invalid)?;
    Ok(reader)
}

pub(super) fn load(path: &Path) -> Result<Option<Database>, GeoIpError> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(failure(error)),
    };
    let modified = file
        .metadata()
        .and_then(|meta| meta.modified())
        .map_err(failure)?;
    let mut bytes = Vec::new();
    file.take(MAX_DATABASE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(failure)?;
    if bytes.len() as u64 > MAX_DATABASE_BYTES {
        return Err(GeoIpError::InvalidDatabase {
            source: Box::new(io::Error::other("database exceeds size limit")),
        });
    }
    Ok(Some(Database {
        reader: validate(bytes)?,
        modified,
    }))
}

pub(super) fn lookup(database: &Database, ip: IpAddr) -> Result<Option<GeoIp>, GeoIpError> {
    let result = database.reader.lookup(ip).map_err(failure)?;
    let Some(city) = result.decode::<geoip2::City>().map_err(failure)? else {
        return Ok(None);
    };
    let coordinates = match (city.location.longitude, city.location.latitude) {
        (Some(longitude), Some(latitude))
            if longitude.is_finite()
                && latitude.is_finite()
                && (-180.0..=180.0).contains(&longitude)
                && (-90.0..=90.0).contains(&latitude) =>
        {
            Some(GeoIpCoordinates {
                longitude,
                latitude,
            })
        }
        _ => None,
    };
    Ok(Some(GeoIp {
        continent_code: city.continent.code.map(str::to_owned),
        country_code: city.country.iso_code.map(str::to_owned),
        country_name: city.country.names.english.map(str::to_owned),
        subdivisions: city
            .subdivisions
            .into_iter()
            .map(|subdivision| GeoIpSubdivision {
                code: subdivision.iso_code.map(str::to_owned),
                name: subdivision.names.english.map(str::to_owned),
            })
            .collect(),
        city_name: city.city.names.english.map(str::to_owned),
        postal_code: city.postal.code.map(str::to_owned),
        coordinates,
        accuracy_radius_km: city.location.accuracy_radius,
        time_zone: city.location.time_zone.map(str::to_owned),
    }))
}
