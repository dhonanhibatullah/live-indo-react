//! Integration tests require REACTION_DATABASE_TEST_DATABASE_URL for a dedicated
//! PostgreSQL/PostGIS test server whose role can create databases and extensions.
//! Each case creates and drops its own database; no existing schema is modified.

use std::{
    path::Path,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use percent_encoding::percent_decode_str;
use reaction_database::{
    database::{self},
    models::Env,
};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use url::Url;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    pool: PgPool,
    admin: PgPool,
    env: Env,
    name: String,
}

impl Fixture {
    async fn new(migrate: bool) -> Option<Self> {
        let Ok(url) = std::env::var("REACTION_DATABASE_TEST_DATABASE_URL") else {
            eprintln!("SKIP database integration: REACTION_DATABASE_TEST_DATABASE_URL is not set");
            return None;
        };
        let parsed = Url::parse(&url).expect("test database URL must be valid");
        let options = PgConnectOptions::from_str(&url).unwrap();
        let admin = PgPoolOptions::new()
            .max_connections(2)
            .connect_with(options.clone())
            .await
            .unwrap();
        let name = format!(
            "live_indo_reaction_test_{}_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        // Identifier is generated solely from an internal prefix and numeric values.
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "CREATE DATABASE {name} TEMPLATE template0"
        )))
        .execute(&admin)
        .await
        .unwrap();
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options.database(&name))
            .await
            .unwrap();
        if migrate {
            database::migrations::run(
                &pool,
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"),
            )
            .await
            .unwrap();
        }
        let decode = |value: &str| {
            percent_decode_str(value)
                .decode_utf8()
                .unwrap()
                .into_owned()
        };
        let env = Env {
            postgres_host: parsed
                .host_str()
                .expect("test URL must use a TCP host")
                .into(),
            postgres_port: parsed.port().unwrap_or(5432),
            postgres_username: if parsed.username().is_empty() {
                "postgres".into()
            } else {
                decode(parsed.username())
            },
            postgres_password: decode(parsed.password().unwrap_or("")),
            postgres_database: name.clone(),
            postgres_ssl_mode: parsed
                .query_pairs()
                .find(|(key, _)| key == "sslmode")
                .map(|(_, value)| value.into_owned())
                .unwrap_or_else(|| "prefer".into()),
        };
        Some(Self {
            pool,
            admin,
            env,
            name,
        })
    }

    fn command(&self, binary: &str, directory: &Path) -> std::process::Command {
        let mut command = std::process::Command::new(binary);
        command
            .current_dir(directory)
            .env_clear()
            .env("REACTION_DATABASE_POSTGRES_HOST", &self.env.postgres_host)
            .env(
                "REACTION_DATABASE_POSTGRES_PORT",
                self.env.postgres_port.to_string(),
            )
            .env(
                "REACTION_DATABASE_POSTGRES_USERNAME",
                &self.env.postgres_username,
            )
            .env(
                "REACTION_DATABASE_POSTGRES_PASSWORD",
                &self.env.postgres_password,
            )
            .env(
                "REACTION_DATABASE_POSTGRES_DATABASE",
                &self.env.postgres_database,
            )
            .env(
                "REACTION_DATABASE_POSTGRES_SSL_MODE",
                &self.env.postgres_ssl_mode,
            );
        command
    }

    async fn count(&self) -> i64 {
        sqlx::query_scalar("SELECT count(*) FROM reactions")
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    fn copy_migrations(&self) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(&self.name);
        let migrations = directory.join("migrations");
        std::fs::create_dir_all(&migrations).unwrap();
        for entry in
            std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")).unwrap()
        {
            let entry = entry.unwrap();
            std::fs::copy(entry.path(), migrations.join(entry.file_name())).unwrap();
        }
        directory
    }

    async fn close(self) {
        self.pool.close().await;
        sqlx::query(sqlx::AssertSqlSafe(format!(
            "DROP DATABASE {} WITH (FORCE)",
            self.name
        )))
        .execute(&self.admin)
        .await
        .unwrap();
        self.admin.close().await;
    }
}

#[tokio::test]
async fn fresh_migration_rerun_and_concurrent_runners_leave_reactions_empty() {
    let Some(fixture) = Fixture::new(false).await else {
        return;
    };
    let second = database::connection::connect_for_migrations(&fixture.env)
        .await
        .unwrap();
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let (first, other) = tokio::join!(
        database::migrations::run(&fixture.pool, &directory),
        database::migrations::run(&second, &directory)
    );
    first.unwrap();
    other.unwrap();
    database::migrations::run(&fixture.pool, &directory)
        .await
        .unwrap();
    let history: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(history, 2);
    assert_eq!(fixture.count().await, 0);
    let connected = database::connection::connect(&fixture.env).await.unwrap();
    connected.close().await;
    second.close().await;
    fixture.close().await;
}

#[tokio::test]
async fn connection_rejects_missing_schema_without_creating_it() {
    let Some(fixture) = Fixture::new(false).await else {
        return;
    };
    let error = database::connection::connect(&fixture.env)
        .await
        .unwrap_err();
    let message = error.to_string();
    for expected in [
        "postgis",
        "reactions",
        "reaction_location_source",
        "apply schema migrations first",
    ] {
        assert!(message.contains(expected));
    }
    let exists: bool = sqlx::query_scalar("SELECT to_regclass('reactions') IS NOT NULL")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert!(!exists);
    fixture.close().await;
}

#[tokio::test]
async fn missing_and_empty_migration_directories_fail_before_schema_changes() {
    let Some(fixture) = Fixture::new(false).await else {
        return;
    };
    let directory = std::env::temp_dir().join(&fixture.name);
    let error = database::migrations::run(&fixture.pool, &directory)
        .await
        .unwrap_err();
    assert_eq!(error.operation, "load schema migrations");
    std::fs::create_dir(&directory).unwrap();
    let error = database::migrations::run(&fixture.pool, &directory)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("contains no migrations"));
    let exists: bool = sqlx::query_scalar("SELECT to_regclass('_sqlx_migrations') IS NOT NULL")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert!(!exists);
    std::fs::remove_dir(directory).unwrap();
    fixture.close().await;
}

#[tokio::test]
async fn executable_migrates_reruns_and_closes_on_transaction_failure_then_recovers() {
    let Some(fixture) = Fixture::new(false).await else {
        return;
    };
    let directory = fixture.copy_migrations();
    let run = || {
        fixture
            .command(env!("CARGO_BIN_EXE_reaction_database"), &directory)
            .output()
            .unwrap()
    };
    for _ in 0..2 {
        let output = run();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(concat!(
            "reaction_database ",
            env!("CARGO_PKG_VERSION"),
            " starting"
        )));
        assert!(
            stdout.find("schema migrations complete").unwrap()
                < stdout.find("database schema check succeeded").unwrap()
        );
        assert!(stdout.contains("database connection pool closed"));
        assert!(stdout.contains("reaction_database completed"));
        assert_eq!(fixture.count().await, 0);
    }
    let failed_file = directory.join("migrations/20261022000100_probe.up.sql");
    std::fs::write(
        &failed_file,
        "CREATE TABLE migration_probe (id INT); SELECT 'private-migration-value'::integer;",
    )
    .unwrap();
    let failed = run();
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("apply schema migrations"));
    assert!(!String::from_utf8_lossy(&failed.stderr).contains("private-migration-value"));
    let stdout = String::from_utf8_lossy(&failed.stdout);
    assert!(stdout.contains("database connection pool closed"));
    assert!(!stdout.contains("reaction_database completed"));
    let exists: bool = sqlx::query_scalar("SELECT to_regclass('migration_probe') IS NOT NULL")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert!(!exists);
    let history: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(history, 2);
    std::fs::write(&failed_file, "CREATE TABLE migration_probe (id INT);").unwrap();
    assert!(run().status.success());
    let history: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(history, 3);
    std::fs::remove_dir_all(directory).unwrap();
    fixture.close().await;
}

#[tokio::test]
async fn executable_rejects_checksum_conflicts_and_schema_drift() {
    let Some(fixture) = Fixture::new(false).await else {
        return;
    };
    let directory = fixture.copy_migrations();
    let run = || {
        fixture
            .command(env!("CARGO_BIN_EXE_reaction_database"), &directory)
            .output()
            .unwrap()
    };
    assert!(run().status.success());
    let path = directory.join("migrations/20261021000200_create_reactions.up.sql");
    let original = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, format!("{original}\n-- edited after application\n")).unwrap();
    let failed = run();
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("apply schema migrations"));
    assert_eq!(fixture.count().await, 0);
    std::fs::write(&path, original).unwrap();
    sqlx::query("DROP TABLE reactions")
        .execute(&fixture.pool)
        .await
        .unwrap();
    let failed = run();
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("missing reactions"));
    assert!(String::from_utf8_lossy(&failed.stdout).contains("database connection pool closed"));
    std::fs::remove_dir_all(directory).unwrap();
    fixture.close().await;
}

#[tokio::test]
async fn valid_reactions_support_all_answers_gifs_ip_families_and_points() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    let first: (i64, bool, String, bool) = sqlx::query_as("INSERT INTO reactions (topic_id,polling_method_code,agree_disagree,request_ip,province_kemendagri_code,location_source) VALUES (42,'agree_disagree',false,'203.0.113.1','31','ip') RETURNING id,agree_disagree,host(request_ip),submitted_at IS NOT NULL AND deleted_at IS NULL")
        .fetch_one(&fixture.pool).await.unwrap();
    assert!(first.0 > 0);
    assert!(!first.1);
    assert_eq!(first.2, "203.0.113.1");
    assert!(first.3);
    for (method, rating, selection) in [
        ("rating", Some(0i16), None),
        ("rating", Some(10), None),
        ("selection", None, Some(0i32)),
        ("selection", None, Some(99)),
        ("future_method", Some(5), None),
    ] {
        sqlx::query("INSERT INTO reactions (topic_id,polling_method_code,rating,selection,request_ip,province_kemendagri_code,location_source) VALUES (42,$1,$2,$3,'2001:db8::1','31','ip')")
            .bind(method).bind(rating).bind(selection).execute(&fixture.pool).await.unwrap();
    }
    let ip: String = sqlx::query_scalar(
        "SELECT host(request_ip) FROM reactions WHERE polling_method_code='future_method'",
    )
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(ip, "2001:db8::1");
    let point: (f64, f64, i32, String, String) = sqlx::query_as("INSERT INTO reactions (topic_id,polling_method_code,selection,comment,gif_provider,gif_id,coordinates,request_ip,province_kemendagri_code,city_kemendagri_code,location_source) VALUES (42,'selection',0,'Comment','klipy','opaque-id',ST_SetSRID(ST_MakePoint(106.8,-6.2),4326),'203.0.113.1','31','31.01','coordinates') RETURNING ST_X(coordinates),ST_Y(coordinates),ST_SRID(coordinates),gif_provider,gif_id")
        .fetch_one(&fixture.pool).await.unwrap();
    assert_eq!(
        point,
        (106.8, -6.2, 4326, "klipy".into(), "opaque-id".into())
    );
    let error = sqlx::query("INSERT INTO reactions (id,topic_id,polling_method_code,rating,request_ip,province_kemendagri_code,location_source) VALUES (999,42,'rating',5,'203.0.113.1','31','ip')").execute(&fixture.pool).await.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("428C9")
    );
    assert_eq!(fixture.count().await, 7);
    fixture.close().await;
}

#[tokio::test]
async fn row_constraints_reject_invalid_answers_gifs_regions_and_coordinates() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    let id: i64 = sqlx::query_scalar("INSERT INTO reactions (topic_id,polling_method_code,agree_disagree,request_ip,province_kemendagri_code,location_source) VALUES (42,'agree_disagree',false,'203.0.113.1','31','ip') RETURNING id").fetch_one(&fixture.pool).await.unwrap();
    for assignment in [
        "agree_disagree=NULL",
        "rating=5",
        "agree_disagree=NULL,rating=-1",
        "agree_disagree=NULL,rating=11",
        "agree_disagree=NULL,selection=-1",
        "polling_method_code=''",
        "topic_id=0",
        "province_kemendagri_code=NULL",
        "province_kemendagri_code='XX'",
        "city_kemendagri_code='3101'",
        "city_kemendagri_code='32.01'",
        "gif_provider='klipy'",
        "gif_id='id'",
        "gif_provider='',gif_id='id'",
        "gif_provider='klipy',gif_id=E'\\t'",
        "location_source='coordinates'",
        "location_source='unknown'",
        "request_ip='not-an-ip'",
        "request_ip=NULL",
        "coordinates=ST_GeomFromText('POINT EMPTY',4326)",
        "coordinates=ST_SetSRID(ST_MakePoint(181,0),4326)",
        "coordinates=ST_SetSRID(ST_MakePoint(0,91),4326)",
        "coordinates=ST_GeomFromText('POINT(NaN 0)',4326)",
        "coordinates=ST_GeomFromText('POINT Z(106 -6 1)',4326)",
        "coordinates=ST_GeomFromText('POINT(106 -6)',3857)",
        "coordinates=ST_GeomFromText('LINESTRING(106 -6,107 -7)',4326)",
    ] {
        // Assignment text is a fixed internal test case; no external input is interpolated.
        let result = sqlx::query(sqlx::AssertSqlSafe(format!(
            "UPDATE reactions SET {assignment} WHERE id=$1"
        )))
        .bind(id)
        .execute(&fixture.pool)
        .await;
        assert!(result.is_err(), "accepted invalid assignment: {assignment}");
    }
    assert_eq!(fixture.count().await, 1);
    fixture.close().await;
}

#[tokio::test]
async fn indexes_support_visible_keyset_feeds_and_spatial_queries() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    let indexes: Vec<(String, String)> = sqlx::query_as(
        "SELECT indexname,indexdef FROM pg_indexes WHERE tablename='reactions' ORDER BY indexname",
    )
    .fetch_all(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(indexes.len(), 5); // Primary key, method, two partial feeds, GiST.
    let definition = |name: &str| &indexes.iter().find(|(index, _)| index == name).unwrap().1;
    assert!(definition("idx_reactions_coordinates_gist").contains("USING gist (coordinates)"));
    assert!(
        definition("idx_reactions_polling_method_code")
            .contains("USING btree (polling_method_code)")
    );
    for index in [
        "idx_reactions_topic_feed",
        "idx_reactions_topic_province_feed",
    ] {
        assert!(definition(index).contains("WHERE (deleted_at IS NULL)"));
    }
    for deleted in [false, false, true] {
        sqlx::query("INSERT INTO reactions (topic_id,polling_method_code,rating,coordinates,request_ip,province_kemendagri_code,location_source,submitted_at,deleted_at) VALUES (42,'rating',5,ST_SetSRID(ST_MakePoint(106.8,-6.2),4326),'203.0.113.1','31','coordinates','2026-10-21T00:00:00Z',CASE WHEN $1 THEN NOW() END)").bind(deleted).execute(&fixture.pool).await.unwrap();
    }
    let first: i64 = sqlx::query_scalar("SELECT id FROM reactions WHERE topic_id=42 AND province_kemendagri_code='31' AND deleted_at IS NULL ORDER BY submitted_at DESC,id DESC LIMIT 1").fetch_one(&fixture.pool).await.unwrap();
    let next: i64 = sqlx::query_scalar("SELECT id FROM reactions WHERE topic_id=42 AND deleted_at IS NULL AND (submitted_at,id)<('2026-10-21T00:00:00Z'::timestamptz,$1) ORDER BY submitted_at DESC,id DESC LIMIT 1").bind(first).fetch_one(&fixture.pool).await.unwrap();
    assert!(first > next);
    let spatial: i64 = sqlx::query_scalar("SELECT count(*) FROM reactions WHERE coordinates && ST_MakeEnvelope(106,-7,107,-6,4326) AND deleted_at IS NULL").fetch_one(&fixture.pool).await.unwrap();
    assert_eq!(spatial, 2);
    let mut connection = fixture.pool.acquire().await.unwrap();
    sqlx::query("SET enable_seqscan=off")
        .execute(&mut *connection)
        .await
        .unwrap();
    let plan: Vec<String> = sqlx::query_scalar(
        "EXPLAIN SELECT id FROM reactions WHERE coordinates && ST_MakeEnvelope(106,-7,107,-6,4326)",
    )
    .fetch_all(&mut *connection)
    .await
    .unwrap();
    assert!(plan.join("\n").contains("idx_reactions_coordinates_gist"));
    connection.close().await.unwrap();
    fixture.close().await;
}

#[tokio::test]
async fn schema_has_no_business_triggers_or_foreign_keys_and_down_migrations_remove_it() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    let foreign_keys: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_constraint WHERE conrelid='reactions'::regclass AND contype='f'",
    )
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(foreign_keys, 0);
    let triggers: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_trigger WHERE tgrelid='reactions'::regclass AND NOT tgisinternal",
    )
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(triggers, 0);
    for migration in [
        include_str!("../migrations/20261021000200_create_reactions.down.sql"),
        include_str!("../migrations/20261021000100_create_extensions.down.sql"),
    ] {
        sqlx::raw_sql(migration)
            .execute(&fixture.pool)
            .await
            .unwrap();
    }
    let removed: bool = sqlx::query_scalar("SELECT to_regclass('reactions') IS NULL AND to_regtype('reaction_location_source') IS NULL AND NOT EXISTS (SELECT 1 FROM pg_extension WHERE extname='postgis')").fetch_one(&fixture.pool).await.unwrap();
    assert!(removed);
    fixture.close().await;
}
