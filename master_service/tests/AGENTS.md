# Master service: tests

Exercise the public `master_service` library without reaching into private
modules. Read the [service guide](../AGENTS.md) and [root guide](../../AGENTS.md)
before adding tests.

## Targets and layout

`Cargo.toml` disables automatic test discovery and defines exactly two test
targets:

- `unit` at `tests/unit/main.rs`
- `integration` at `tests/integration/main.rs`

Each `main.rs` contains module declarations only. Tests live in focused modules
that mirror the implemented source layer and capability.

```text
tests/
  unit/
    main.rs
    domain/
    infrastructure/
    application/          # when unit tests for usecases exist
    presentation/         # when unit tests for transport mapping exist
  integration/
    main.rs
    support/              # fixtures and test doubles, never test cases
    infrastructure/
      repository/
    presentation/         # when real transport behavior exists
    composition/          # when process lifecycle exists
```

Do not create empty `mod.rs` files for future layers. Add a module only when its
first test exists.

## Unit tests

`tests/unit/` contains black-box unit tests of public behavior. Cargo compiles
them as a separate test crate, so they cannot access `pub(crate)` items.

- `unit/domain/`: pure model, error-code, value-invariant, and contract tests.
- `unit/infrastructure/`: adapters tested with in-memory fakes.
- `unit/application/`: future usecase tests using fake domain contracts.
- `unit/presentation/`: future DTO, mapping, and safe-error tests.

Unit tests must not connect to a database, open sockets, start processes, mutate
environment variables, or depend on wall-clock timing. Prefer deterministic
inputs and capturing test doubles. Keep one capability per file and one observable
behavior per test.

Private helpers remain covered by inline `#[cfg(test)]` modules beside the code.
Do not widen visibility or import source files with `#[path]` merely to move those
tests into `tests/unit/`.

## Integration tests

`tests/integration/` verifies real boundary behavior. Group tests by the outermost
boundary being exercised:

- `infrastructure/repository/`: PostgreSQL/PostGIS repository behavior.
- `presentation/`: future transport and wire-contract behavior.
- `composition/`: future startup, dependency wiring, and shutdown behavior.

Do not add integration tests for application usecases; application behavior uses
fakes in `unit/application/`. Domain values do not need external integration
tests.

Reusable fixtures belong under `support/` and must contain no `#[test]` functions.
Keep support APIs `pub(crate)` and as narrow as possible. Repository integration
tests require `MASTER_DATABASE_TEST_DATABASE_URL` for a PostgreSQL/PostGIS server
whose role can create databases and extensions. Each test must:

1. Create a uniquely named isolated database.
2. Apply `../master_database/migrations/*.up.sql` in sorted order.
3. Drop the database after closing its pools.

When the variable is absent, print a clear skip message and return. Report skipped
database tests as unverified, not passing database behavior. Never modify a
developer's live database.

## Doubles and assertions

Keep test doubles outside production modules. Shared integration doubles belong in
`tests/integration/support/`; unit-test doubles may stay local to their capability
module until reuse justifies sharing.

Assert observable behavior and classified errors rather than implementation
details. Preserve checks for stable error codes, safe messages, SQLSTATE and
constraint metadata, source preservation, transaction commit/rollback, and log
redaction where applicable. Avoid sleeps, execution-order dependencies, and tests
that merely duplicate an implementation expression.

## Verification

Run from this directory:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test unit
cargo test
```

Run database integration tests with the test URL explicitly set:

```sh
MASTER_DATABASE_TEST_DATABASE_URL=... cargo test --test integration -- --nocapture
```
