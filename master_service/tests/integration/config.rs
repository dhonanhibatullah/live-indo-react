use std::{error::Error, fs, io};

use master_service::config::geoip_conf::{LoadError, load_from_path};
use uuid::Uuid;

#[test]
fn geoip_configuration_loads_explicit_file_and_preserves_read_failures() {
    let directory = std::env::temp_dir().join(format!("master-geoip-conf-{}", Uuid::new_v4()));
    fs::create_dir(&directory).unwrap();
    let path = directory.join("GeoIP.conf");
    let result = (|| {
        fs::write(
            &path,
            "AccountID 123\nLicenseKey fixture-key\nEditionIDs GeoLite2-City\n",
        )?;
        let conf = load_from_path(&path)?;
        assert_eq!(conf.account_id, 123);
        assert_eq!(conf.license_key, "fixture-key");
        assert_eq!(conf.edition_ids, ["GeoLite2-City"]);

        fs::write(&path, b"LicenseKey \xff")?;
        assert!(matches!(load_from_path(&path), Err(LoadError::Io(_))));
        fs::remove_file(&path)?;
        let error = match load_from_path(&path) {
            Ok(_) => panic!("missing file should fail"),
            Err(error) => error,
        };
        let source = error.source().unwrap().downcast_ref::<io::Error>().unwrap();
        assert_eq!(source.kind(), io::ErrorKind::NotFound);
        assert!(!format!("{error:?}").contains(&directory.to_string_lossy().to_string()));
        Ok::<_, Box<dyn Error>>(())
    })();
    fs::remove_dir_all(directory).unwrap();
    result.unwrap();
}
