use std::{
    path::Path,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use pgdt::{
    Pgdt,
    sqlx::{self, AssertSqlSafe, PgPool, postgres::PgConnectOptions, postgres::PgPoolOptions},
};

static NEXT_DATABASE: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Fixture {
    pool: PgPool,
    admin: PgPool,
    name: String,
}

impl Fixture {
    pub(crate) async fn new() -> Option<Self> {
        let Ok(url) = std::env::var("MASTER_DATABASE_TEST_DATABASE_URL") else {
            eprintln!("SKIP repository integration: MASTER_DATABASE_TEST_DATABASE_URL is not set");
            return None;
        };
        let options = PgConnectOptions::from_str(&url).expect("test database URL must be valid");
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options.clone())
            .await
            .expect("connect to test database server");
        let name = format!(
            "live_indo_repository_test_{}_{}_{}",
            std::process::id(),
            NEXT_DATABASE.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock must be after the Unix epoch")
                .as_nanos()
        );
        sqlx::query(AssertSqlSafe(format!(
            "CREATE DATABASE {name} TEMPLATE template0"
        )))
        .execute(&admin)
        .await
        .expect("create isolated test database");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options.database(&name))
            .await
            .expect("connect to isolated test database");
        apply_migrations(&pool).await;

        Some(Self { pool, admin, name })
    }

    pub(crate) fn database(&self) -> Pgdt {
        Pgdt::new(self.pool.clone())
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(crate) async fn polling_method_id(&self, code: &str) -> i64 {
        sqlx::query_scalar("SELECT id FROM polling_methods WHERE code = $1")
            .bind(code)
            .fetch_one(&self.pool)
            .await
            .expect("seeded polling method must exist")
    }

    pub(crate) async fn close(self) {
        self.pool.close().await;
        sqlx::query(AssertSqlSafe(format!(
            "DROP DATABASE {} WITH (FORCE)",
            self.name
        )))
        .execute(&self.admin)
        .await
        .expect("drop isolated test database");
        self.admin.close().await;
    }
}

async fn apply_migrations(pool: &PgPool) {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../master_database/migrations");
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .expect("read master database migrations")
        .map(|entry| entry.expect("read migration entry").path())
        .filter(|path| path.to_string_lossy().ends_with(".up.sql"))
        .collect();
    paths.sort();
    for path in paths {
        let sql = std::fs::read_to_string(&path).expect("read migration");
        sqlx::raw_sql(AssertSqlSafe(sql))
            .execute(pool)
            .await
            .unwrap_or_else(|error| panic!("apply {}: {error}", path.display()));
    }
}
