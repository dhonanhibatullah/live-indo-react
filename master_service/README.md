# master_service

Master-data service for live-indo-react. It serves topics, polling methods, Indonesian
provinces, cities/regencies, coordinate lookups, and the national boundary over HTTP
and gRPC. Participation remains anonymous; the service has no authentication.

## Requirements

- Rust 1.94 or newer
- PostgreSQL with PostGIS and pg_trgm
- MaxMind GeoIP.conf containing AccountID, LicenseKey and GeoLite2-City among EditionIDs
- A database prepared by [`master_database`](../master_database/README.md)

Migrations and seeds are owned by `master_database`. This service does not modify
the schema.

## Run locally

```sh
cd master_service
cp .env.example .env
# edit the PostgreSQL credentials
# place your private MaxMind GeoIP.conf in this directory
cargo run
```

Configuration is read from `.env` and the process environment; process variables
win. See [`.env.example`](.env.example) for all available settings.

Default listeners:

- HTTP: `0.0.0.0:8080`, routes under `/v1`
- gRPC: `0.0.0.0:8081`

Every request needs an `x-trace-id` metadata header containing a non-zero UUID.
`x-actor` is optional correlation metadata, not authentication.

## Daily GeoIP refresh

The launcher wires driver -> infrastructure -> application -> presentation.
Driver construction opens PostgreSQL; infrastructure initializes the GeoIP
reader. Once all wiring is complete, startup awaits the existing maintenance
usecase and verifies that an MMDB is loaded before starting listeners or cron.
If initialization fails or exceeds ten minutes, startup fails and the pool closes. A current local MMDB is reused without downloading.

The in-process cron presentation then calls maintenance daily at **00:00 UTC**.
It creates a fresh context, avoids overlapping executions, and has a ten-minute
execution limit. Failures/timeouts wait for the next daily run. Only application
and infrastructure emit operation logs; there are no cron job-level logs.

Set MASTER_SERVICE_GEOIP_CONF_PATH (default `GeoIP.conf`) and
MASTER_SERVICE_GEOIP_DATABASE_PATH (default `data/GeoLite2-City.mmdb`) in `.env`
when using other paths. Relative paths use the process working directory.
Configuration is required at startup. A missing/corrupt MMDB is downloaded and
replaced during the initial refresh. The daily scheduler waits until the next
00:00 UTC after initialization, avoiding a second immediate refresh.

Shutdown stops new runs and drains running work within the service shutdown
limit. Each instance refreshes its own MMDB; shared-file writers require external
coordination. MaxMind credentials and downloaded production data stay out of Git
and the Docker build context.

## Docker

Build from the repository root because the service uses the shared Rust workspace:

```sh
docker build -f master_service/Dockerfile -t live-indo-master-service .
docker run --env-file master_service/.env \
  --mount "type=bind,source=$(pwd)/master_service/GeoIP.conf,target=/app/GeoIP.conf,readonly" \
  --mount "type=bind,source=$(pwd)/master_service/data,target=/app/data" \
  -p 8080:8080 -p 8081:8081 live-indo-master-service
```

Create the host data directory before running. The container runs as UID/GID
10001:10001; provision the directory as writable and the credential file as
readable for that user. Override the two path settings to match other mounts.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Database integration tests run when `MASTER_DATABASE_TEST_DATABASE_URL` points to a
disposable PostgreSQL/PostGIS server. Otherwise those cases are skipped.
