# Reaction database

PostgreSQL/PostGIS storage for anonymous, topic-scoped reactions in live-indo-react.
This Rust tool applies tracked schema migrations and checks the resulting schema.
It inserts no reactions or seed data; submissions belong to `reaction_service`.

See the [ERD](docs/erd.dbml) for the data design and [AGENTS.md](AGENTS.md) for
contributor guidance.

## Setup

You need Rust 1.94 or newer, a pre-created PostgreSQL database, and PostGIS installed
on its server. Live integration tests use PostgreSQL 17 and PostGIS 3.5.

From the repository root:

```sh
cd reaction_database
cp .env.example .env
```

Replace `CHANGE_ME` credentials in `.env`. The migration role needs permission to
connect and create PostGIS and schema objects. The database must already exist;
the tool does not create it or install server packages.

| Variable | Default / requirement |
| --- | --- |
| `REACTION_DATABASE_POSTGRES_HOST` | `127.0.0.1` |
| `REACTION_DATABASE_POSTGRES_PORT` | `5432`; integer from 1 to 65535 |
| `REACTION_DATABASE_POSTGRES_USERNAME` | Required; replace `CHANGE_ME` |
| `REACTION_DATABASE_POSTGRES_PASSWORD` | Required; replace `CHANGE_ME`; an explicitly empty value is accepted |
| `REACTION_DATABASE_POSTGRES_DATABASE` | `reaction_database` |
| `REACTION_DATABASE_POSTGRES_SSL_MODE` | `prefer`; accepts `disable`, `allow`, `prefer`, `require`, `verify-ca`, `verify-full` |

An optional `.env` is loaded from the **working directory**. Process environment
variables override file settings. Passwords are preserved exactly. App name and
version come from Cargo metadata at build time.

## Run migrations

From this directory:

```sh
cargo run --release --locked
```

The executable connects, applies two migrations (PostGIS, then reactions), verifies
PostGIS/table/enum prerequisites, closes the pool, and exits. Successful fresh and
repeated runs leave `reactions` empty. There is no seeder, seed history, or demo
catalog. INFO goes to stdout, safe error descriptions to stderr; failures return a
nonzero status. The pool is closed after migration or schema-verification failure.

SQL files are loaded from the working directory's `migrations/` folder. SQLx
serializes migration execution, records versions/checksums in `_sqlx_migrations`,
skips completed versions, and runs each pending migration transactionally. Earlier
committed migrations survive a later failure; fix the pending migration's cause
and rerun. Do not edit applied files or rewrite history to bypass a conflict.
A manually created schema needs an explicit baseline before using this runner.
Paired down migrations are available for intentional rollback in reverse order;
the executable applies only up migrations.

For deployment, run `cargo build --release --locked` and distribute
`target/release/reaction_database` with a `migrations/` directory in its working
directory and runtime environment settings.

## Docker

Build and run from this directory:

```sh
docker build -t live-indo-reaction-database .
docker run --rm --env-file .env \
  -e REACTION_DATABASE_POSTGRES_HOST=YOUR_DATABASE_HOST \
  live-indo-reaction-database
```

Use a host reachable from the container. For a database container, add
`--network NETWORK_NAME` and use its network name as the host. `127.0.0.1` inside
the migration container refers to that container itself.

The image runs the single executable directly as UID/GID `10001:10001`, applies
migrations, checks the schema, and exits. PostgreSQL/PostGIS runs separately; this
image does not contain a server or expose a network service. Credentials are
supplied at runtime. `.env` files and local build artifacts are excluded from the
build context.

## Data and service boundaries

The table stores one required polling answer plus optional comment/GIF reference,
a required Indonesian province code, optional city/regency code and WGS84 point,
client IP, original submission time, and soft-deletion time.

- `polling_method_code` is indexed text. Exactly one of boolean agreement, rating
  (0–10), or nonnegative zero-based selection index is populated. The service
  validates the actual topic method and selection bounds.
- GIFs use paired `gif_provider`/`gif_id` fields. There is no stored rendition URL
  or selected provider integration in this tool.
- Coordinates use `geometry(Point,4326)` and a GiST index: longitude X, latitude Y.
  `INET` accepts IPv4/IPv6. Exact coordinates and raw IPs stay internal and must not
  appear in public feeds or broadcasts.
- Topic IDs and Kemendagri codes reference master data across a database boundary;
  there are no SQL foreign keys or local master-table copies. `reaction_service`
  validates references and rejects IP countries outside Indonesia or unresolved
  countries, as well as submissions without a resolved Indonesian province.
- The service owns original-payload immutability, content/provider validation,
  normalization, and limits. No business triggers enforce these policies here.
- The two visible-feed indexes use `WHERE deleted_at IS NULL`. Feed queries must
  include that predicate and use `(submitted_at, id)` keyset pagination. There are
  no extra standalone IP or deletion-time indexes and no partitioning assumption.

The schema does not implement user identity, rate limiting, aggregates, or atomic
reaction-to-broadcast delivery. PostGIS geometry distances in SRID 4326 are degrees;
metre-based queries require appropriate geography/projected-CRS handling and an
index matching that query.

## Development checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

To run live integration cases, set `REACTION_DATABASE_TEST_DATABASE_URL` to a
**dedicated disposable PostgreSQL/PostGIS server** before running `cargo test`.
Its role must be able to create/drop databases and create PostGIS extensions.
Each case creates and removes an independent database. Without this variable,
live cases are skipped; configuration/model/unit tests still run.

The integration suite covers executable reruns, concurrent migrations, checksum
conflicts, rollback/recovery, schema constraints, GIF pairs, IPv4/IPv6, PostGIS
points and GiST usage, partial feed indexes, keyset pagination, and down migrations.
