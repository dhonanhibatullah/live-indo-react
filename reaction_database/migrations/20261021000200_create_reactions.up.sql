CREATE TYPE reaction_location_source AS ENUM ('coordinates', 'ip');

CREATE TABLE reactions (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    topic_id BIGINT NOT NULL,
    polling_method_code TEXT NOT NULL,
    agree_disagree BOOLEAN,
    rating SMALLINT,
    selection INTEGER,
    comment TEXT,
    gif_provider TEXT,
    gif_id TEXT,
    coordinates geometry(Point,4326),
    request_ip INET NOT NULL,
    province_kemendagri_code VARCHAR(2) NOT NULL,
    city_kemendagri_code VARCHAR(5),
    location_source reaction_location_source NOT NULL,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    CONSTRAINT ck_reactions_gif_reference_pair
        CHECK ((gif_provider IS NULL) = (gif_id IS NULL)),
    CONSTRAINT ck_reactions_gif_provider_not_blank
        CHECK (gif_provider IS NULL OR gif_provider !~ '^[[:space:]]*$'),
    CONSTRAINT ck_reactions_gif_id_not_blank
        CHECK (gif_id IS NULL OR gif_id !~ '^[[:space:]]*$'),
    CONSTRAINT ck_reactions_topic_id_positive
        CHECK (topic_id > 0),
    CONSTRAINT ck_reactions_exactly_one_answer
        CHECK (num_nonnulls(agree_disagree, rating, selection) = 1),
    CONSTRAINT ck_reactions_rating_range
        CHECK (rating IS NULL OR rating BETWEEN 0 AND 10),
    CONSTRAINT ck_reactions_selection_index_nonnegative
        CHECK (selection IS NULL OR selection >= 0),
    CONSTRAINT ck_reactions_polling_method_not_blank
        CHECK (length(btrim(polling_method_code)) > 0),
    CONSTRAINT ck_reactions_coordinates_valid
        CHECK (coordinates IS NULL OR (
            NOT ST_IsEmpty(coordinates) AND ST_IsValid(coordinates)
            AND ST_X(coordinates) BETWEEN -180 AND 180
            AND ST_Y(coordinates) BETWEEN -90 AND 90
        )),
    CONSTRAINT ck_reactions_coordinate_source
        CHECK (location_source <> 'coordinates' OR coordinates IS NOT NULL),
    CONSTRAINT ck_reactions_province_code_format
        CHECK (province_kemendagri_code ~ '^[0-9]{2}$'),
    CONSTRAINT ck_reactions_city_code_format
        CHECK (city_kemendagri_code IS NULL OR city_kemendagri_code ~ '^[0-9]{2}[.][0-9]{2}$'),
    CONSTRAINT ck_reactions_city_province_prefix
        CHECK (city_kemendagri_code IS NULL OR substring(city_kemendagri_code FROM 1 FOR 2) = province_kemendagri_code)
);

CREATE INDEX idx_reactions_polling_method_code ON reactions (polling_method_code);
CREATE INDEX idx_reactions_topic_feed ON reactions (topic_id, submitted_at, id)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_reactions_topic_province_feed
    ON reactions (topic_id, province_kemendagri_code, submitted_at, id)
    WHERE deleted_at IS NULL;
CREATE INDEX idx_reactions_coordinates_gist ON reactions USING GIST (coordinates);

COMMENT ON TABLE reactions IS
    'Anonymous topic-scoped reactions. External master references and business validation belong to reaction_service. No production seed rows are inserted.';
COMMENT ON COLUMN reactions.polling_method_code IS
    'Flexible method-code snapshot. The service validates compatibility with the populated answer column.';
COMMENT ON COLUMN reactions.selection IS
    'Zero-based topic option index. The service checks bounds and preserves topic option ordering after responses exist.';
COMMENT ON COLUMN reactions.coordinates IS
    'Internal user-supplied 2D WGS84 point: longitude X, latitude Y. Never expose through public feeds or broadcasts.';
COMMENT ON COLUMN reactions.request_ip IS
    'Internal service-resolved client host IP. Never expose through public feeds or broadcasts.';
COMMENT ON COLUMN reactions.gif_id IS
    'Opaque provider content ID paired with gif_provider, not a rendition URL.';
COMMENT ON COLUMN reactions.submitted_at IS
    'Server-assigned original submission time. Accepted payloads are immutable by service policy.';
