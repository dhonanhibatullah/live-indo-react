use std::{collections::BTreeSet, fs, path::Path};

use master_database::models::{
    Boundary, CatalogSeedFile, City, CityKind, Env, IndonesiaBoundary, PollingMethod, Province,
    SeedFile,
};
use serde_json::json;

#[test]
fn models_deserialize_all_seed_files_without_losing_province_links_or_provenance() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds");
    let mut files: Vec<_> = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();

    let mut provinces = BTreeSet::new();
    let mut province_files = 0;
    let mut city_files = 0;
    let mut boundary_files = 0;
    let mut cities = 0;
    let mut city_count = 0;
    let mut regency_count = 0;

    for (index, path) in files.iter().enumerate() {
        let name = path.file_stem().unwrap().to_str().unwrap();
        let (prefix, expected_name) = name.split_once('_').unwrap();
        let version = prefix.parse::<i64>().unwrap();
        assert_eq!(version, index as i64 + 1);
        let bytes = fs::read(path).unwrap();
        if expected_name.starts_with("provinces_") {
            let seed: SeedFile<Province> = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(seed.version, version);
            assert_eq!(seed.name, expected_name);
            assert_eq!(seed.srid, 4326);
            assert_eq!(seed.source.copyright.license, "MIT");
            assert_eq!(seed.source.revision.len(), 40);
            assert_eq!(seed.source.sha256.len(), 64);
            assert_eq!(
                seed.source.generated_at.date_naive().to_string(),
                "2026-02-16"
            );
            for record in seed.records {
                assert!(provinces.insert(record.kemendagri_code));
                assert!(record.boundary_reference_date.is_none());
                assert!(!record.boundary_source.is_empty());
                let Boundary::MultiPolygon { coordinates } = record.boundary;
                assert!(!coordinates.is_empty());
            }
            province_files += 1;
        } else if expected_name == "indonesia_boundary" {
            let seed: SeedFile<IndonesiaBoundary> = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(seed.version, 47);
            assert_eq!(seed.name, expected_name);
            assert_eq!(seed.srid, 4326);
            assert_eq!(seed.source.copyright.license, "CC BY 4.0");
            assert_eq!(seed.source.revision.len(), 40);
            assert_eq!(seed.source.sha256.len(), 64);
            assert_eq!(seed.records.len(), 1);
            let record = &seed.records[0];
            assert!(!record.boundary_source.is_empty());
            let Boundary::MultiPolygon { coordinates } = &record.boundary;
            assert!(!coordinates.is_empty());
            boundary_files += 1;
        } else if expected_name == "polling_methods" {
            let seed: CatalogSeedFile<PollingMethod> = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(seed.version, 46);
            assert_eq!(seed.name, expected_name);
            assert_eq!(seed.records.len(), 3);
        } else {
            assert!(expected_name.starts_with("cities_"));
            let seed: SeedFile<City> = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(seed.version, version);
            assert_eq!(seed.name, expected_name);
            assert_eq!(seed.srid, 4326);
            assert_eq!(seed.source.publisher.name, "Laravel Nusa");
            assert!(!seed.source.normalization.is_empty());
            assert!(!seed.source.license_file.is_empty());
            for record in seed.records {
                assert!(provinces.contains(&record.province_kemendagri_code));
                assert!(
                    record
                        .kemendagri_code
                        .starts_with(&format!("{}.", record.province_kemendagri_code))
                );
                match record.kind {
                    CityKind::City => city_count += 1,
                    CityKind::Regency => regency_count += 1,
                }
                cities += 1;
            }
            city_files += 1;
        }
    }

    assert_eq!((province_files, city_files, boundary_files), (7, 38, 1));
    assert_eq!((provinces.len(), cities), (38, 514));
    assert_eq!((city_count, regency_count), (98, 416));
}

#[test]
fn geography_models_preserve_wire_shape_and_reject_wrong_geometry_or_dimension() {
    let geometry = json!({
        "type": "MultiPolygon",
        "coordinates": [[[[100.0, 0.0], [101.0, 0.0], [101.0, 1.0], [100.0, 0.0]]]],
    });
    let boundary: Boundary = serde_json::from_value(geometry.clone()).unwrap();
    assert_eq!(serde_json::to_value(boundary).unwrap(), geometry);
    assert!(
        serde_json::from_value::<Boundary>(json!({
            "type": "Polygon", "coordinates": [],
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<Boundary>(json!({
            "type": "MultiPolygon", "coordinates": [[[[100.0, 0.0, 5.0]]]],
        }))
        .is_err()
    );
    assert_eq!(
        serde_json::from_str::<CityKind>("\"city\"").unwrap(),
        CityKind::City
    );
    assert_eq!(
        serde_json::from_str::<CityKind>("\"regency\"").unwrap(),
        CityKind::Regency
    );
    assert!(serde_json::from_str::<CityKind>("\"district\"").is_err());
}

#[test]
fn boundary_reference_dates_are_calendar_dates_and_can_be_unknown() {
    let record = json!({
        "kemendagri_code": "11",
        "name": "Test province",
        "boundary": {"type": "MultiPolygon", "coordinates": []},
        "boundary_source": "test source",
        "boundary_reference_date": "2024-02-29",
    });
    let parsed: Province = serde_json::from_value(record.clone()).unwrap();
    assert_eq!(
        parsed.boundary_reference_date.unwrap().to_string(),
        "2024-02-29"
    );

    let mut invalid = record.clone();
    invalid["boundary_reference_date"] = json!("2025-02-29");
    assert!(serde_json::from_value::<Province>(invalid).is_err());

    let mut unknown = record;
    unknown["boundary_reference_date"] = json!(null);
    assert!(
        serde_json::from_value::<Province>(unknown)
            .unwrap()
            .boundary_reference_date
            .is_none()
    );
}

#[test]
fn environment_debug_output_redacts_the_password() {
    let settings = Env {
        postgres_host: "localhost".into(),
        postgres_port: 5432,
        postgres_username: "test-user".into(),
        postgres_password: "do-not-print-this-password".into(),
        postgres_database: "master_test".into(),
        postgres_ssl_mode: "require".into(),
    };
    for debug in [format!("{settings:?}"), format!("{settings:#?}")] {
        assert!(!debug.contains(&settings.postgres_password));
        assert!(debug.contains("[REDACTED]"));
    }
}
