# pgdt

Context-aware PostgreSQL execution and transactions using native SQLx types.
This crate follows Go's `pgxdt` pattern: ordinary contexts use the pool; contexts
carrying a transaction use that transaction's connection. Nested transaction
callbacks reuse the outer transaction.

The crate owns no service context, SQL builder, row model, domain error, or logger.
It forwards SQLx queries and errors and returns native PostgreSQL rows and results.
It requires Tokio and supports PostgreSQL only.

## Context and usage

Implement `TransactionContext` on your service context. The method attaching the
handle must clone the context while preserving actor, trace, and other metadata.

```no_run
use futures_util::TryStreamExt;
use pgdt::{Pgdt, Transaction, TransactionContext, sqlx::{self, PgPool, Row}};

#[derive(Clone, Default)]
struct Context {
    actor: Option<String>,
    trace_id: Option<String>,
    transaction: Option<Transaction>,
}

impl TransactionContext for Context {
    fn transaction(&self) -> Option<&Transaction> {
        self.transaction.as_ref()
    }

    fn with_transaction(&self, transaction: Transaction) -> Self {
        Self { transaction: Some(transaction), ..self.clone() }
    }
}

async fn example(pool: PgPool) -> Result<(), sqlx::Error> {
    let database = Pgdt::new(pool);
    let transactor = database.transactor();
    let context = Context::default();

    // No transaction: run through the pool with separately bound parameters.
    let row = database.query_optional(
        &context,
        sqlx::query("SELECT id FROM items WHERE name = $1").bind("example"),
    ).await?;

    // The callback gets a derived context; pass it to every repository operation.
    let database_ref = &database;
    let id = transactor.with_tx(&context, |context| async move {
        let row = database_ref.query_row(
            &context,
            sqlx::query("INSERT INTO items (name) VALUES ($1) RETURNING id")
                .bind("example"),
        ).await?;
        Ok::<_, sqlx::Error>(row.try_get::<i64, _>("id")?)
    }).await?;

    // Stream rather than buffering all results.
    let mut rows = database.query(&context, sqlx::query("SELECT id FROM items"));
    while let Some(row) = rows.try_next().await? {
        let id: i64 = row.try_get("id")?;
    }
    Ok(())
}
```

`exec` returns `PgQueryResult`, including `rows_affected()`. `query_row` returns
`sqlx::Error::RowNotFound` when no row exists; `query_optional` returns `None`.
`Pgdt::with_tx` and `Transactor::with_tx` have identical behavior. Callbacks may
return any `Send` result and error implementing `From<sqlx::Error>`.

Construct one `Pgdt` and clone it for repositories and transactors. Independently
constructed wrappers have different transaction identities, even for clones of
the same pool. A foreign or inactive handle returns `sqlx::Error::Protocol`; it
never falls back to the pool. Use `pool()` for composition-level shutdown or
operations that intentionally bypass context routing.

## Transaction lifetime

Successful callbacks commit; failed callbacks roll back and preserve their
original error. Begin and commit failures remain native SQLx errors. Secondary
rollback errors are discarded when preserving a callback error. Nested callbacks
create no savepoints and perform no independent commit or rollback: propagate an
inner error to the owner if its work should be rolled back. A PostgreSQL statement
failure can still abort the transaction independently of Rust error handling.

Concurrent transaction operations serialize on one connection. A query stream
acquires its connection lazily and holds it until exhausted or dropped. **Consume
or drop the stream before awaiting another operation on that transaction or
completing its callback**, otherwise completion may wait indefinitely.

Panic or future cancellation immediately invalidates the handle. Cleanup releases
the transaction or schedules rollback after in-flight operations release their
lock. Retained context clones cannot start new queries after scope completion;
retained streams must still be released for cleanup to finish. Panics propagate.
There are no automatic retries or crate-generated logs.

## Features

- Default: `tls-rustls-ring`, using SQLx's Rustls/ring backend with WebPKI roots.
- Optional: `json`, `time`, `uuid`, and `macros`, forwarding SQLx type/macro support.
- `--no-default-features` supports connections without TLS.

Consumers build their queries using SQLx or a compatible query builder such as
SeaQuery. This crate does not rebuild SQL or parameters. Raw SQL must come from
trusted code; bind input values separately.

## Tests

From the workspace root, normal tests and documentation examples run with:

```sh
cargo test --workspace
```

Database tests are ignored by default. Use a disposable PostgreSQL 18 database;
the fixtures create separate schemas and exercise backend termination. Never
point the test URL at a development or production database.

```sh
docker run --detach --rm --name pgdt-test \
  -e POSTGRES_PASSWORD=pgdt-test-only -e POSTGRES_DB=pgdt_test \
  -p 127.0.0.1:35432:5432 postgres:18
# Wait until this succeeds:
docker exec pgdt-test pg_isready -U postgres -d pgdt_test

PGDT_TEST_DATABASE_URL='postgres://postgres:pgdt-test-only@127.0.0.1:35432/pgdt_test' \
  cargo test --workspace --all-features -- --include-ignored

docker stop pgdt-test
```

Explicitly running database tests without `PGDT_TEST_DATABASE_URL` fails with
an explanatory message. Optional type tests run when their features are enabled.
