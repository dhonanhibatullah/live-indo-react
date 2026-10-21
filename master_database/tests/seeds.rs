use std::{collections::BTreeSet, error::Error, fs, path::Path};

use master_database::{
    models::City,
    seeds::{LoadError, cities, polling_methods, provinces},
};
use sha2::{Digest, Sha256};

#[test]
fn compressed_assets_preserve_exact_source_bytes_and_history_checksums() {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("seeds");
    let mut original_total = 0;
    let mut compressed_total = 0;
    let files = provinces::FILES
        .iter()
        .chain(cities::FILES)
        .chain(polling_methods::FILES);

    for (index, file) in files.enumerate() {
        assert_eq!(file.version, index as i64 + 1);
        let original = fs::read(directory.join(file.filename)).unwrap();
        assert_eq!(file.json().unwrap().as_bytes(), original);
        assert_eq!(
            file.checksum_sha256,
            Sha256::digest(&original)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        original_total += original.len();
        compressed_total += file.compressed_len();
    }

    assert!(compressed_total < original_total);
    eprintln!(
        "Seed payload: {original_total} original bytes -> {compressed_total} embedded gzip bytes"
    );
}

#[test]
fn typed_groups_load_in_order_without_runtime_file_reads() {
    let mut province_codes = BTreeSet::new();
    let mut city_codes = BTreeSet::new();
    let mut province_files = 0;
    let mut city_files = 0;

    for result in provinces::load() {
        let loaded = result.unwrap();
        assert_eq!(loaded.file.version, loaded.data.version);
        assert_eq!(
            loaded.file.filename,
            format!("{:04}_{}.json", loaded.data.version, loaded.data.name)
        );
        assert_eq!(loaded.data.srid, 4326);
        for record in loaded.data.records {
            assert!(province_codes.insert(record.kemendagri_code));
        }
        province_files += 1;
    }
    for result in cities::load() {
        let loaded = result.unwrap();
        assert_eq!(loaded.file.version, loaded.data.version);
        assert_eq!(
            loaded.file.filename,
            format!("{:04}_{}.json", loaded.data.version, loaded.data.name)
        );
        assert_eq!(loaded.data.srid, 4326);
        for record in loaded.data.records {
            assert!(province_codes.contains(&record.province_kemendagri_code));
            assert!(city_codes.insert(record.kemendagri_code));
        }
        city_files += 1;
    }

    assert_eq!((province_files, city_files), (7, 38));
    assert_eq!((province_codes.len(), city_codes.len()), (38, 514));
}

#[test]
fn wrong_record_type_returns_a_named_deserialization_error() {
    let file = &provinces::FILES[0];
    let error = file.load::<City>().unwrap_err();
    assert!(matches!(error, LoadError::Deserialize { .. }));
    assert!(error.to_string().contains(file.filename));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<serde_json::Error>()
            .is_some()
    );
}

#[test]
fn polling_methods_use_a_catalog_envelope_and_load_exactly_three_behaviors() {
    let loaded = polling_methods::load().next().unwrap().unwrap();
    assert_eq!(loaded.data.version, 46);
    assert_eq!(loaded.data.name, "polling_methods");
    assert_eq!(loaded.file.filename, "0046_polling_methods.json");
    let codes: Vec<_> = loaded
        .data
        .records
        .iter()
        .map(|record| record.code.as_str())
        .collect();
    assert_eq!(codes, ["agree_disagree", "rating", "selection"]);
    assert!(
        loaded
            .data
            .records
            .iter()
            .all(|record| !record.name.is_empty() && !record.description.is_empty())
    );
    let json: serde_json::Value = serde_json::from_str(&loaded.file.json().unwrap()).unwrap();
    assert!(json.get("srid").is_none());
    assert!(json.get("source").is_none());
    assert!(
        loaded
            .file
            .load::<master_database::models::Province>()
            .is_err()
    );
}
