# master_service

Master-data service for live-indo-react. It serves topics, polling methods, Indonesian
provinces, cities/regencies, coordinate lookups, and the national boundary over HTTP
and gRPC. Participation remains anonymous; the service has no authentication.

## Requirements

- Rust 1.94 or newer
- PostgreSQL with PostGIS and pg_trgm
- A database prepared by [`master_database`](../master_database/README.md)

Migrations and seeds are owned by `master_database`. This service does not modify
the schema.

## Run locally

```sh
cd master_service
cp .env.example .env
# edit the PostgreSQL credentials
cargo run
```

Configuration is read from `.env` and the process environment; process variables
win. See [`.env.example`](.env.example) for all available settings.

Default listeners:

- HTTP: `0.0.0.0:8080`, routes under `/v1`
- gRPC: `0.0.0.0:8081`

Every request needs an `x-trace-id` metadata header containing a non-zero UUID.
`x-actor` is optional correlation metadata, not authentication.

## Docker

Build from the repository root because the service uses the shared Rust workspace:

```sh
docker build -f master_service/Dockerfile -t live-indo-master-service .
docker run --env-file master_service/.env -p 8080:8080 -p 8081:8081 live-indo-master-service
```

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Database integration tests run when `MASTER_DATABASE_TEST_DATABASE_URL` points to a
disposable PostgreSQL/PostGIS server. Otherwise those cases are skipped.
