use std::{
    fs::{self, FileTimes},
    io::{self, Read, Write},
    path::Path,
    sync::Arc,
    time::SystemTime,
};

use flate2::read::GzDecoder;
use tokio::sync::{OwnedMutexGuard, RwLock};

use crate::domain::models::error::GeoIpError;

use super::reader::{Database, MAX_DATABASE_BYTES, failure, validate};

fn invalid(source: impl std::error::Error + Send + Sync + 'static) -> GeoIpError {
    GeoIpError::InvalidDownload {
        source: Box::new(source),
    }
}

fn invalid_message(message: &'static str) -> GeoIpError {
    invalid(io::Error::other(message))
}

/// Read only the expected regular file; never unpack archive-controlled paths.
fn extract(bytes: &[u8]) -> Result<Vec<u8>, GeoIpError> {
    let decoder = GzDecoder::new(bytes).take(MAX_DATABASE_BYTES + 1);
    let mut archive = tar::Archive::new(decoder);
    let mut database = None;
    for (count, entry) in archive.entries().map_err(invalid)?.enumerate() {
        if count >= 10_000 {
            return Err(invalid_message("too many archive entries"));
        }
        let mut entry = entry.map_err(invalid)?;
        let path = entry.path().map_err(invalid)?;
        if path
            .file_name()
            .is_none_or(|name| name != "GeoLite2-City.mmdb")
        {
            continue;
        }
        if !entry.header().entry_type().is_file() || database.is_some() {
            return Err(invalid_message("expected one regular City database"));
        }
        if entry.size() > MAX_DATABASE_BYTES {
            return Err(invalid_message("database exceeds size limit"));
        }
        let mut content = Vec::new();
        entry.read_to_end(&mut content).map_err(invalid)?;
        database = Some(content);
    }
    // Consume remaining gzip data to verify its trailer/checksum and expansion bound.
    let mut decoder = archive.into_inner();
    io::copy(&mut decoder, &mut io::sink()).map_err(invalid)?;
    if decoder.limit() == 0 {
        return Err(invalid_message("archive expansion exceeds size limit"));
    }
    database.ok_or_else(|| invalid_message("archive has no City database"))
}

/// Holds the update permit even if the requesting future is cancelled. A complete
/// blocking installation may finish; the previous database is kept on every error.
pub(super) fn install(
    archive: Vec<u8>,
    modified: SystemTime,
    path: &Path,
    state: &RwLock<Option<Arc<Database>>>,
    _permit: OwnedMutexGuard<()>,
) -> Result<(), GeoIpError> {
    let bytes = extract(&archive)?;
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(failure)?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent).map_err(failure)?;
    temporary.write_all(&bytes).map_err(failure)?;
    let reader = validate(bytes)?;
    temporary
        .as_file()
        .set_times(FileTimes::new().set_modified(modified))
        .map_err(failure)?;
    temporary.as_file().sync_all().map_err(failure)?;
    let mut current = state.blocking_write();
    temporary
        .persist(path)
        .map_err(|error| failure(error.error))?;
    *current = Some(Arc::new(Database { reader, modified }));
    Ok(())
}
