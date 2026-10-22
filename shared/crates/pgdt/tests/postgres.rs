mod common;

use std::{future::pending, time::Duration};

use futures_util::TryStreamExt;
use pgdt::{
    Pgdt, Transaction, TransactionContext, Transactor,
    sqlx::{self, Row},
};
use tokio::sync::oneshot;

use common::{Context, context, count, database, insert};

#[derive(Debug)]
enum Error {
    Driver(sqlx::Error),
    Business(&'static str),
}
impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Self::Driver(error)
    }
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn native_queries_binding_streaming_and_empty_results() {
    let owned_db = database(2).await;
    let db = &owned_db;
    let ctx = &context();
    let value = "private input'; DROP TABLE items; --".to_owned();
    let result = db
        .exec(
            ctx,
            sqlx::query("INSERT INTO items VALUES ($1, $2)")
                .bind(1_i64)
                .bind(&value),
        )
        .await
        .unwrap();
    assert_eq!(result.rows_affected(), 1);
    let row = db
        .query_row(
            ctx,
            sqlx::query("SELECT id, value FROM items WHERE id = $1").bind(1_i64),
        )
        .await
        .unwrap();
    assert_eq!(row.try_get::<String, _>("value").unwrap(), value);
    let mut builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT ");
    builder.push_bind(&value).push("::TEXT AS value");
    let row = db.query_row(ctx, builder.build()).await.unwrap();
    assert_eq!(row.try_get::<String, _>("value").unwrap(), value);

    let rows: Vec<_> = db
        .query(ctx, sqlx::query("SELECT generate_series(1, 3) AS n"))
        .try_collect()
        .await
        .unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[2].try_get::<i32, _>("n").unwrap(), 3);
    assert!(
        db.query_optional(ctx, sqlx::query("SELECT id FROM items WHERE FALSE"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        db.query_row(ctx, sqlx::query("SELECT id FROM items WHERE FALSE"))
            .await,
        Err(sqlx::Error::RowNotFound)
    ));
    let rows: Vec<_> = db
        .query(ctx, sqlx::query("SELECT id FROM items WHERE FALSE"))
        .try_collect()
        .await
        .unwrap();
    assert!(rows.is_empty());
    let error = db
        .exec(
            ctx,
            sqlx::query("INSERT INTO items VALUES (1, 'duplicate')"),
        )
        .await
        .unwrap_err();
    let source = error.as_database_error().unwrap();
    assert_eq!(source.code().as_deref(), Some("23505"));
    assert_eq!(source.constraint(), Some("items_pkey"));
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn primitive_types_bytes_and_typed_nulls_use_native_decoding() {
    let owned_db = database(1).await;
    let db = &owned_db;
    let ctx = &context();
    let row = db.query_row(ctx, sqlx::query("SELECT $1::BOOL AS b, $2::INT2 AS s, $3::INT4 AS i, $4::INT8 AS l, $5::FLOAT4 AS f, $6::FLOAT8 AS d, $7::BYTEA AS bytes, $8::TEXT AS nullable")
        .bind(true).bind(2_i16).bind(3_i32).bind(4_i64).bind(1.25_f32).bind(2.5_f64).bind(vec![0_u8,255]).bind(None::<String>)).await.unwrap();
    assert!(row.try_get::<bool, _>("b").unwrap());
    assert_eq!(row.try_get::<i16, _>("s").unwrap(), 2);
    assert_eq!(row.try_get::<i32, _>("i").unwrap(), 3);
    assert_eq!(row.try_get::<i64, _>("l").unwrap(), 4);
    assert_eq!(row.try_get::<f32, _>("f").unwrap(), 1.25);
    assert_eq!(row.try_get::<f64, _>("d").unwrap(), 2.5);
    assert_eq!(row.try_get::<Vec<u8>, _>("bytes").unwrap(), vec![0, 255]);
    assert_eq!(row.try_get::<Option<String>, _>("nullable").unwrap(), None);
    assert!(row.try_get::<String, _>("nullable").is_err());
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn commit_preserves_context_results_and_invalidates_retained_handles() {
    let owned_db = database(2).await;
    let db = &owned_db;
    let ctx = &context();
    let (send, receive) = oneshot::channel();
    let result: Result<i64, Error> = db
        .with_tx(ctx, |tx| async move {
            assert_eq!(tx.actor, ctx.actor);
            assert_eq!(tx.trace_id, ctx.trace_id);
            let handle = tx.transaction().unwrap();
            assert!(handle.is_active());
            assert_eq!(handle, &handle.clone());
            insert(db, &tx, 1).await?;
            assert_eq!(count(db, &tx).await, 1);
            assert_eq!(count(db, ctx).await, 0);
            send.send(tx).unwrap();
            Ok(42)
        })
        .await;
    assert_eq!(result.unwrap(), 42);
    let retained = receive.await.unwrap();
    assert!(!retained.transaction().unwrap().is_active());
    assert!(matches!(
        insert(db, &retained, 2).await,
        Err(sqlx::Error::Protocol(_))
    ));
    assert!(matches!(
        db.transactor()
            .with_tx(&retained, |_| async { Ok::<_, sqlx::Error>(()) })
            .await,
        Err(sqlx::Error::Protocol(_))
    ));
    assert_eq!(count(db, ctx).await, 1);
    assert!(ctx.transaction().is_none());
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn callback_and_statement_failures_roll_back_preserving_errors() {
    let owned_db = database(1).await;
    let db = &owned_db;
    let ctx = &context();
    let result: Result<(), Error> = db
        .with_tx(ctx, |tx| async move {
            insert(db, &tx, 1).await?;
            Err(Error::Business("original"))
        })
        .await;
    assert!(matches!(result, Err(Error::Business("original"))));
    assert_eq!(count(db, ctx).await, 0);
    let result: Result<(), Error> = db
        .with_tx(ctx, |tx| async move {
            insert(db, &tx, 2).await?;
            insert(db, &tx, 2).await?;
            Ok(())
        })
        .await;
    assert!(
        matches!(result, Err(Error::Driver(error)) if error.as_database_error().unwrap().code().as_deref() == Some("23505"))
    );
    assert_eq!(count(db, ctx).await, 0);
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn nested_calls_reuse_owner_without_independent_commit_or_rollback() {
    let owned_db = database(1).await;
    let db = &owned_db;
    let ctx = &context();
    let transactor = Transactor::new(db.clone());
    let result: Result<(), Error> = transactor
        .with_tx(ctx, |outer| async move {
            let expected = outer.clone();
            let nested: Result<(), Error> = db
                .with_tx(&outer, |inner| async move {
                    assert_eq!(inner, expected);
                    insert(db, &inner, 1).await?;
                    Err(Error::Business("nested"))
                })
                .await;
            assert!(matches!(nested, Err(Error::Business("nested"))));
            insert(db, &outer, 2).await?;
            Ok(())
        })
        .await;
    result.unwrap();
    assert_eq!(count(db, ctx).await, 2);
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn foreign_wrappers_are_rejected_and_concurrent_queries_serialize() {
    let owned_db = database(1).await;
    let db = &owned_db;
    let other = Pgdt::new(db.pool().clone());
    let ctx = &context();
    let result: Result<(), sqlx::Error> = db
        .with_tx(ctx, |tx| async move {
            assert!(matches!(
                insert(&other, &tx, 9).await,
                Err(sqlx::Error::Protocol(_))
            ));
            assert!(matches!(
                other
                    .with_tx(&tx, |_| async { Ok::<_, sqlx::Error>(()) })
                    .await,
                Err(sqlx::Error::Protocol(_))
            ));
            let (a, b) = tokio::join!(insert(db, &tx, 1), insert(db, &tx, 2));
            a?;
            b?;
            Ok(())
        })
        .await;
    result.unwrap();
    assert_eq!(count(db, ctx).await, 2);
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn streaming_is_lazy_and_drop_releases_transaction_lock() {
    let owned_db = database(1).await;
    let db = &owned_db;
    let ctx = &context();
    let result: Result<(), sqlx::Error> = db
        .with_tx(ctx, |tx| async move {
            // An unpolled stream has not acquired the transaction lock.
            let unpolled = db.query(&tx, sqlx::query("SELECT 1"));
            insert(db, &tx, 1).await?;
            drop(unpolled);
            let mut rows = db.query(&tx, sqlx::query("SELECT generate_series(1, 10) AS n"));
            assert!(rows.try_next().await?.is_some());
            assert!(
                tokio::time::timeout(Duration::from_millis(30), insert(db, &tx, 2))
                    .await
                    .is_err()
            );
            drop(rows);
            insert(db, &tx, 2).await?;
            Ok(())
        })
        .await;
    result.unwrap();
    assert_eq!(count(db, ctx).await, 2);
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn panic_and_cancellation_release_transaction_with_escaped_contexts() {
    let owned_db = database(1).await;
    let db = &owned_db;
    for panic in [false, true] {
        let task_db = db.clone();
        let (send, receive) = oneshot::channel();
        let task = tokio::spawn(async move {
            let task_db = &task_db;
            task_db
                .with_tx(&context(), |tx| async move {
                    insert(task_db, &tx, 1).await?;
                    send.send(tx).unwrap();
                    if panic {
                        panic!("transaction callback panic");
                    }
                    pending::<()>().await;
                    Ok::<_, sqlx::Error>(())
                })
                .await
        });
        let retained = receive.await.unwrap();
        if !panic {
            task.abort();
        }
        assert_eq!(task.await.unwrap_err().is_panic(), panic);
        assert!(!retained.transaction().unwrap().is_active());
        assert!(matches!(
            insert(db, &retained, 2).await,
            Err(sqlx::Error::Protocol(_))
        ));
        assert_eq!(count(db, &Context::default()).await, 0);
    }
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn cancellation_waits_for_in_flight_work_then_rolls_back() {
    let owned_db = database(2).await;
    let db = &owned_db;
    let task_db = db.clone();
    let (send, receive) = oneshot::channel();
    let task = tokio::spawn(async move {
        let task_db = &task_db;
        task_db
            .with_tx(&context(), |tx| async move {
                insert(task_db, &tx, 1).await?;
                let query_db = task_db.clone();
                let query_ctx = tx.clone();
                let query = tokio::spawn(async move {
                    query_db
                        .exec(&query_ctx, sqlx::query("SELECT pg_sleep(1)"))
                        .await
                });
                send.send((tx, query)).unwrap();
                pending::<()>().await;
                Ok::<_, sqlx::Error>(())
            })
            .await
    });
    let (retained, query) = receive.await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let row = db.query_row(&Context::default(), sqlx::query("SELECT COUNT(*) AS count FROM pg_stat_activity WHERE query = 'SELECT pg_sleep(1)' AND state = 'active'")).await.unwrap();
            if row.try_get::<i64,_>("count").unwrap()>0 { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }).await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert!(!retained.transaction().unwrap().is_active());
    query.await.unwrap().unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while count(db, &Context::default()).await != 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn deferred_commit_failure_preserves_constraint_and_closes_handle() {
    let owned_db = database(1).await;
    let db = &owned_db;
    let ctx = &context();
    db.exec(
        ctx,
        sqlx::raw_sql("CREATE TABLE deferred (id BIGINT UNIQUE DEFERRABLE INITIALLY DEFERRED)"),
    )
    .await
    .unwrap();
    let (send, receive) = oneshot::channel();
    let result: Result<(), sqlx::Error> = db
        .with_tx(ctx, |tx| async move {
            db.exec(&tx, sqlx::query("INSERT INTO deferred VALUES (1), (1)"))
                .await?;
            send.send(tx).unwrap();
            Ok(())
        })
        .await;
    let error = result.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23505")
    );
    let retained = receive.await.unwrap();
    assert!(!retained.transaction().unwrap().is_active());
    let row = db
        .query_row(ctx, sqlx::query("SELECT COUNT(*) AS count FROM deferred"))
        .await
        .unwrap();
    assert_eq!(row.try_get::<i64, _>("count").unwrap(), 0);
    db.pool().close().await;
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn lost_connection_rollback_keeps_business_error_and_closed_pool_begin_fails() {
    let owned_db = database(2).await;
    let db = &owned_db;
    let ctx = &context();
    let result: Result<(), Error> = db
        .with_tx(ctx, |tx| async move {
            let row = db
                .query_row(&tx, sqlx::query("SELECT pg_backend_pid() AS pid"))
                .await?;
            let pid = row.try_get::<i32, _>("pid")?;
            let row = db
                .query_row(
                    ctx,
                    sqlx::query("SELECT pg_terminate_backend($1) AS terminated").bind(pid),
                )
                .await?;
            assert!(row.try_get::<bool, _>("terminated")?);
            Err(Error::Business("original"))
        })
        .await;
    assert!(matches!(result, Err(Error::Business("original"))));
    db.pool().close().await;
    assert!(matches!(
        db.with_tx(ctx, |_| async { Ok::<_, sqlx::Error>(()) })
            .await,
        Err(sqlx::Error::PoolClosed)
    ));
}

#[derive(Clone)]
struct BrokenContext;
impl TransactionContext for BrokenContext {
    fn transaction(&self) -> Option<&Transaction> {
        None
    }
    fn with_transaction(&self, _: Transaction) -> Self {
        self.clone()
    }
}

#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn faulty_context_implementation_does_not_leak_transaction() {
    let owned_db = database(1).await;
    let db = &owned_db;
    assert!(matches!(
        db.with_tx(&BrokenContext, |_| async { Ok::<_, sqlx::Error>(()) })
            .await,
        Err(sqlx::Error::Protocol(_))
    ));
    assert_eq!(count(db, &Context::default()).await, 0);
    db.pool().close().await;
}

#[tokio::test]
async fn public_futures_are_send_and_support_borrowed_inputs_without_connecting() {
    fn assert_send<T: Send>(_: T) {}
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://localhost/mate_test")
        .unwrap();
    let db = Pgdt::new(pool);
    let ctx = context();
    let input = "borrowed input".to_owned();
    assert_send(db.exec(&ctx, sqlx::query("SELECT $1::TEXT").bind(&input)));
    assert_send(db.query(&ctx, sqlx::query("SELECT $1::TEXT").bind(&input)));
    assert_send(db.query_row(&ctx, sqlx::query("SELECT $1::TEXT").bind(&input)));
    assert_send(db.query_optional(&ctx, sqlx::query("SELECT $1::TEXT").bind(&input)));
    assert_send(db.with_tx(&ctx, |context| async move { Ok::<_, sqlx::Error>(context) }));
    db.pool().close().await;
}

#[cfg(feature = "json")]
#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn json_feature_round_trips_native_values_and_null() {
    let db = database(1).await;
    let ctx = context();
    let value = serde_json::json!({"private": [1, true]});
    let row = db
        .query_row(
            &ctx,
            sqlx::query("SELECT $1::JSONB AS value, $2::JSONB AS nullable")
                .bind(&value)
                .bind(None::<serde_json::Value>),
        )
        .await
        .unwrap();
    assert_eq!(row.try_get::<serde_json::Value, _>("value").unwrap(), value);
    assert_eq!(
        row.try_get::<Option<serde_json::Value>, _>("nullable")
            .unwrap(),
        None
    );
    db.pool().close().await;
}

#[cfg(feature = "time")]
#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn time_feature_round_trips_native_values_and_null() {
    let db = database(1).await;
    let ctx = context();
    let value = time::OffsetDateTime::UNIX_EPOCH;
    let row = db
        .query_row(
            &ctx,
            sqlx::query("SELECT $1::TIMESTAMPTZ AS value, $2::TIMESTAMPTZ AS nullable")
                .bind(value)
                .bind(None::<time::OffsetDateTime>),
        )
        .await
        .unwrap();
    assert_eq!(
        row.try_get::<time::OffsetDateTime, _>("value").unwrap(),
        value
    );
    assert_eq!(
        row.try_get::<Option<time::OffsetDateTime>, _>("nullable")
            .unwrap(),
        None
    );
    db.pool().close().await;
}

#[cfg(feature = "uuid")]
#[tokio::test]
#[ignore = "requires PGDT_TEST_DATABASE_URL"]
async fn uuid_feature_round_trips_native_values_and_null() {
    let db = database(1).await;
    let ctx = context();
    let value = uuid::Uuid::from_u128(42);
    let row = db
        .query_row(
            &ctx,
            sqlx::query("SELECT $1::UUID AS value, $2::UUID AS nullable")
                .bind(value)
                .bind(None::<uuid::Uuid>),
        )
        .await
        .unwrap();
    assert_eq!(row.try_get::<uuid::Uuid, _>("value").unwrap(), value);
    assert_eq!(
        row.try_get::<Option<uuid::Uuid>, _>("nullable").unwrap(),
        None
    );
    db.pool().close().await;
}
