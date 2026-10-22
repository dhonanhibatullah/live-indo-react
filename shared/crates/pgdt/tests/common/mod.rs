use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use pgdt::{
    Pgdt, Transaction, TransactionContext,
    sqlx::{self, PgPool, Row, postgres::PgPoolOptions},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Context {
    pub actor: Option<String>,
    pub trace_id: Option<String>,
    pub transaction: Option<Transaction>,
}

impl TransactionContext for Context {
    fn transaction(&self) -> Option<&Transaction> {
        self.transaction.as_ref()
    }
    fn with_transaction(&self, transaction: Transaction) -> Self {
        Self {
            transaction: Some(transaction),
            ..self.clone()
        }
    }
}

pub fn context() -> Context {
    Context {
        actor: Some("test actor".into()),
        trace_id: Some("test trace".into()),
        ..Context::default()
    }
}

pub async fn database(max_connections: u32) -> Pgdt {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let url = std::env::var("PGDT_TEST_DATABASE_URL")
        .expect("set PGDT_TEST_DATABASE_URL to an isolated PostgreSQL database");
    let schema = format!(
        "pgdt_{}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let bootstrap = PgPool::connect(&url).await.unwrap();
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!("CREATE SCHEMA {schema}")))
        .execute(&bootstrap)
        .await
        .unwrap();
    bootstrap.close().await;
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(2))
        .after_connect(move |connection, _| {
            let sql = format!("SET search_path TO {schema}");
            Box::pin(async move {
                sqlx::raw_sql(sqlx::AssertSqlSafe(sql))
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect(&url)
        .await
        .unwrap();
    let database = Pgdt::new(pool);
    database
        .exec(
            &Context::default(),
            sqlx::raw_sql("CREATE TABLE items (id BIGINT PRIMARY KEY, value TEXT UNIQUE)"),
        )
        .await
        .unwrap();
    database
}

pub async fn insert(database: &Pgdt, context: &Context, id: i64) -> Result<(), sqlx::Error> {
    database
        .exec(
            context,
            sqlx::query("INSERT INTO items VALUES ($1, $2)")
                .bind(id)
                .bind(format!("item-{id}")),
        )
        .await?;
    Ok(())
}

pub async fn count(database: &Pgdt, context: &Context) -> i64 {
    database
        .query_row(context, sqlx::query("SELECT COUNT(*) AS count FROM items"))
        .await
        .unwrap()
        .try_get("count")
        .unwrap()
}
