use super::*;

static CORRUPT: EmbeddedSeed = EmbeddedSeed {
    version: 1,
    filename: "0001_corrupt.json",
    checksum_sha256: "",
    compressed: b"not gzip data",
};

#[test]
fn corrupt_compressed_data_returns_a_named_error() {
    use std::error::Error;

    let error = CORRUPT.json().unwrap_err();
    assert!(matches!(error, LoadError::Decompress { .. }));
    assert!(error.to_string().contains("0001_corrupt.json"));
    assert!(error.source().is_some());
}
