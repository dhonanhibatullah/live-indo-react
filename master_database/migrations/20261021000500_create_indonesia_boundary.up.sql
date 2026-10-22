CREATE TABLE indonesia_boundary (
    id SMALLINT PRIMARY KEY DEFAULT 1,
    boundary geometry(MultiPolygon,4326) NOT NULL,
    boundary_source TEXT NOT NULL,
    boundary_reference_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,

    CONSTRAINT ck_indonesia_boundary_singleton CHECK (id = 1)
);

CREATE INDEX idx_indonesia_boundary_gist ON indonesia_boundary USING GIST (boundary);

COMMENT ON TABLE indonesia_boundary IS
    'Singleton national boundary used for country containment, including Indonesian waters outside province boundaries.';
COMMENT ON COLUMN indonesia_boundary.boundary IS
    '2D WGS84 MultiPolygon; X is longitude and Y is latitude. Topological validity is not enforced by the database.';
COMMENT ON COLUMN indonesia_boundary.boundary_source IS
    'Dataset/publisher reference, version or release identifier, and attribution.';
COMMENT ON COLUMN indonesia_boundary.boundary_reference_date IS
    'Effective/reference date of the source boundary, not the import timestamp.';
