CREATE TABLE seed_history (
    version BIGINT PRIMARY KEY,
    filename TEXT NOT NULL,
    checksum_sha256 VARCHAR(64) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX uq_seed_history_filename ON seed_history (filename);

COMMENT ON TABLE seed_history IS
    'Successfully applied seed files. Insert the history row in the same transaction as its data writes.';
COMMENT ON COLUMN seed_history.version IS
    'Positive numeric filename prefix; MAX(version) identifies the latest successfully applied seed.';
COMMENT ON COLUMN seed_history.checksum_sha256 IS
    'Lowercase hex SHA-256 of the exact seed file bytes; detect edits to already applied seeds.';
