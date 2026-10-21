# Master database

PostgreSQL/PostGIS reference data for live-indo-react: nationwide reaction topics,
Indonesian provinces, and cities/regencies with geographic boundaries. The Rust
executable imports 38 provinces and 514 cities/regencies from 45 embedded seed
files. Topics have a schema but no initial seed data.

See the [ERD](docs/erd.dbml) for the schema and [AGENTS.md](AGENTS.md) for contributor
guidance.

## Setup

You need Rust 1.94 or newer, PostgreSQL with PostGIS and pg_trgm available on the
server, and a pre-created target database. Live tests have been verified with
PostgreSQL 17 and PostGIS 3.5.

Run commands from this directory:

```sh
cd master_database
cp .env.example .env
```

Edit `.env` with your database credentials. Create the target database before
applying migrations.

| Variable | Default / requirement |
| --- | --- |
| `MASTER_DATABASE_POSTGRES_HOST` | `127.0.0.1` |
| `MASTER_DATABASE_POSTGRES_PORT` | `5432`; integer from 1 to 65535 |
| `MASTER_DATABASE_POSTGRES_USERNAME` | Required; replace `CHANGE_ME` |
| `MASTER_DATABASE_POSTGRES_PASSWORD` | Required; replace `CHANGE_ME` (an explicitly empty value is accepted) |
| `MASTER_DATABASE_POSTGRES_DATABASE` | `master_database` |
| `MASTER_DATABASE_POSTGRES_SSL_MODE` | `prefer`; accepts `disable`, `allow`, `prefer`, `require`, `verify-ca`, `verify-full` |

The executable reads an optional `.env` from its **working directory**. Process
environment variables override file values. App name/version come from Cargo
metadata at build time.

## Run migrations and seeds

Run the complete workflow from this directory:

```sh
cargo run --release --locked
```

The executable connects, applies pending SQL migrations, verifies the schema,
seeds provinces followed by cities, and closes the pool. The configured role
needs permission to create the PostGIS/pg_trgm extensions and schema objects,
read and insert reference and history rows, and use identity sequences. The target database must already
exist; both extensions must be available on its server. Name searches on topics,
provinces, and cities have GIN trigram indexes.

SQL files are read from the working directory's `migrations/` folder. SQLx records
schema versions and checksums in `_sqlx_migrations`, locks migration execution,
and skips applied versions. Each pending migration runs in a transaction. A
migration failure stops the workflow before seeding and returns a failing exit
status after closing the pool. `seed_history` separately tracks JSON imports.

A schema previously created manually requires an explicit SQLx baseline before
using this workflow; it cannot safely infer which SQL files were applied. Paired
`.down.sql` files are available for intentional rollback in reverse order; the
executable applies only up migrations.

## Docker

Build and run from this directory:

```sh
docker build -t live-indo-master-database .
docker run --rm --env-file .env \
  -e MASTER_DATABASE_POSTGRES_HOST=YOUR_DATABASE_HOST \
  live-indo-master-database
```

Replace `YOUR_DATABASE_HOST` with a host reachable from the container. For a
database container, attach to its Docker network with `--network NETWORK_NAME`
and use the database container's network name. `127.0.0.1` inside this container
refers to the container itself.

The image includes the schema files and a single compiled executable. Its
entrypoint runs migrations first, then seeds, and exits; a migration failure stops
the container before seeding. Reruns check both schema and seed history. PostgreSQL
with PostGIS and pg_trgm available must be running separately, and the configured role must have both
migration and seed privileges. No database server is included in the image.

The runtime runs as UID/GID `10001:10001`. Credentials are supplied at runtime;
`.env` files and build artifacts are excluded from the build context.

## Seed behavior

After migrations, the executable imports provinces before cities and exits. A fresh successful
run inserts 552 geographic rows and 45 seed-history rows. INFO messages go to
stdout; errors go to stderr with a nonzero exit status.

Run the same command again safely: matching completed files are skipped, and
existing regions are preserved by Kemendagri code, including inactive or
soft-deleted records. Each file and its history entry commit together. If a file
fails, fix the cause and rerun; earlier files remain committed. Cooperating seed
runners serialize work with a database advisory lock.

Editing an already applied seed, even its formatting, causes a checksum conflict.
Use a new seed version and an explicit data-update implementation for corrections;
this importer does not overwrite existing records.

For deployment, build with `cargo build --release --locked` and run
`target/release/master_database` with the environment configured and the
`migrations/` directory in its working directory. Seed JSON is
compressed into the executable at build time, then decoded one file at a time;
the source `seeds/` directory is not needed at runtime.

## Development checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

To include live database and executable integration cases, set
`MASTER_DATABASE_TEST_DATABASE_URL` to a dedicated disposable PostgreSQL/PostGIS
test server before `cargo test`. Its role must be able to create/drop databases
and create PostGIS extensions. Each case creates and removes its own database;
without the variable, live cases are skipped. Other tests run without a database.

The geographic seed envelopes record provenance and source revisions. Retain
their attribution and the [upstream license](seeds/LICENSE.laravel-nusa) when
redistributing the data.
