//! Integration tests require MASTER_DATABASE_TEST_DATABASE_URL for a dedicated
//! PostgreSQL/PostGIS test server whose role can create databases and extensions.
//! Each case creates and drops its own database; no existing schema is modified.

use std::{
    path::Path,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use master_database::{
    database::{self, SeedSummary},
    models::Env,
};
use percent_encoding::percent_decode_str;
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
        let Ok(url) = std::env::var("MASTER_DATABASE_TEST_DATABASE_URL") else {
            eprintln!("SKIP database integration: MASTER_DATABASE_TEST_DATABASE_URL is not set");
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
            "live_indo_test_{}_{}_{}",
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
            let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
            let mut paths: Vec<_> = std::fs::read_dir(directory)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.to_string_lossy().ends_with(".up.sql"))
                .collect();
            paths.sort();
            for path in paths {
                sqlx::raw_sql(sqlx::AssertSqlSafe(std::fs::read_to_string(path).unwrap()))
                    .execute(&pool)
                    .await
                    .unwrap();
            }
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
            .env("MASTER_DATABASE_POSTGRES_HOST", &self.env.postgres_host)
            .env(
                "MASTER_DATABASE_POSTGRES_PORT",
                self.env.postgres_port.to_string(),
            )
            .env(
                "MASTER_DATABASE_POSTGRES_USERNAME",
                &self.env.postgres_username,
            )
            .env(
                "MASTER_DATABASE_POSTGRES_PASSWORD",
                &self.env.postgres_password,
            )
            .env(
                "MASTER_DATABASE_POSTGRES_DATABASE",
                &self.env.postgres_database,
            )
            .env(
                "MASTER_DATABASE_POSTGRES_SSL_MODE",
                &self.env.postgres_ssl_mode,
            );
        command
    }

    async fn count(&self, table: &str) -> i64 {
        assert!(["provinces", "cities", "seed_history", "polling_methods"].contains(&table));
        sqlx::query_scalar(match table {
            "provinces" => "SELECT count(*) FROM provinces",
            "cities" => "SELECT count(*) FROM cities",
            "seed_history" => "SELECT count(*) FROM seed_history",
            "polling_methods" => "SELECT count(*) FROM polling_methods",
            _ => unreachable!(),
        })
        .fetch_one(&self.pool)
        .await
        .unwrap()
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
async fn executable_migrates_then_seeds_and_rolls_back_failed_sql() {
    let Some(fixture) = Fixture::new(false).await else {
        return;
    };
    let directory = std::env::temp_dir().join(&fixture.name);
    let migrations = directory.join("migrations");
    std::fs::create_dir_all(&migrations).unwrap();
    for entry in
        std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")).unwrap()
    {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), migrations.join(entry.file_name())).unwrap();
    }
    let run = || {
        fixture
            .command(env!("CARGO_BIN_EXE_master_database"), &directory)
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
        assert!(
            stdout.find("schema migrations complete").unwrap()
                < stdout.find("seed run started").unwrap()
        );
        assert_eq!(fixture.count("provinces").await, 38);
        assert_eq!(fixture.count("cities").await, 514);
        assert_eq!(fixture.count("seed_history").await, 46);
        let count: i64 = sqlx::query_scalar("SELECT count(*) FROM _sqlx_migrations WHERE success")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
        assert_eq!(count, 6);
    }
    let connected = database::connection::connect(&fixture.env).await.unwrap();
    connected.close().await;

    std::fs::write(
        migrations.join("20261022000100_failure.up.sql"),
        "CREATE TABLE rollback_probe (id INT); SELECT 'sensitive-migration-value'::integer;",
    )
    .unwrap();
    let failed = run();
    assert!(!failed.status.success());
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("apply schema migrations"));
    assert!(!stderr.contains("sensitive-migration-value"));
    let stdout = String::from_utf8_lossy(&failed.stdout);
    assert!(!stdout.contains("seed run started"));
    assert!(stdout.contains("database connection pool closed"));
    let exists: bool = sqlx::query_scalar("SELECT to_regclass('rollback_probe') IS NOT NULL")
        .fetch_one(&fixture.pool)
        .await
        .unwrap();
    assert!(!exists);

    std::fs::remove_file(migrations.join("20261022000100_failure.up.sql")).unwrap();
    let applied = migrations.join("20261021000200_create_topics.up.sql");
    let mut contents = std::fs::read_to_string(&applied).unwrap();
    contents.push_str("\n-- changed after application\n");
    std::fs::write(applied, contents).unwrap();
    assert!(!run().status.success());
    std::fs::remove_dir_all(directory).unwrap();
    fixture.close().await;
}

#[tokio::test]
async fn executable_seeds_reruns_and_reports_schema_failure() {
    let Some(fixture) = Fixture::new(false).await else {
        return;
    };
    let directory = std::env::temp_dir().join(&fixture.name);
    let migrations = directory.join("migrations");
    std::fs::create_dir_all(&migrations).unwrap();
    for entry in
        std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations")).unwrap()
    {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), migrations.join(entry.file_name())).unwrap();
    }
    let run = || {
        fixture
            .command(env!("CARGO_BIN_EXE_master_database"), &directory)
            .output()
            .unwrap()
    };
    let first = run();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let stdout = String::from_utf8_lossy(&first.stdout);
    assert!(stdout.contains(concat!(
        "master_database ",
        env!("CARGO_PKG_VERSION"),
        " starting"
    )));
    assert!(stdout.contains("files_applied=46 files_skipped=0 rows_inserted=555"));
    assert!(stdout.contains("database connection pool closed"));
    assert!(stdout.contains("master_database completed"));
    assert_eq!(fixture.count("provinces").await, 38);
    assert_eq!(fixture.count("cities").await, 514);
    assert_eq!(fixture.count("seed_history").await, 46);

    let second = run();
    assert!(second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).contains("files_applied=0 files_skipped=46"));

    sqlx::query("DROP TABLE cities")
        .execute(&fixture.pool)
        .await
        .unwrap();
    let failed = run();
    assert!(!failed.status.success());
    assert!(String::from_utf8_lossy(&failed.stderr).contains("apply schema migrations first"));
    assert!(!String::from_utf8_lossy(&failed.stdout).contains("master_database completed"));

    std::fs::remove_dir_all(directory).unwrap();
    fixture.close().await;
}

#[tokio::test]
async fn fresh_run_and_rerun_preserve_seed_history_and_geometry() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    let connected = database::connection::connect(&fixture.env).await.unwrap();
    assert_eq!(
        database::seed(&connected).await.unwrap(),
        SeedSummary {
            files_applied: 46,
            files_skipped: 0,
            rows_inserted: 555,
            rows_skipped: 0
        }
    );
    assert_eq!(fixture.count("provinces").await, 38);
    assert_eq!(fixture.count("cities").await, 514);
    assert_eq!(fixture.count("seed_history").await, 46);
    let invalid: i64 = sqlx::query_scalar("SELECT count(*) FROM (SELECT boundary FROM provinces UNION ALL SELECT boundary FROM cities) b WHERE NOT ST_IsValid(boundary) OR ST_IsEmpty(boundary) OR ST_SRID(boundary) <> 4326 OR GeometryType(boundary) <> 'MULTIPOLYGON'").fetch_one(&connected).await.unwrap();
    assert_eq!(invalid, 0);
    let mismatches: i64 = sqlx::query_scalar("SELECT count(*) FROM cities c JOIN provinces p ON p.id=c.province_id WHERE left(c.kemendagri_code,2) <> p.kemendagri_code").fetch_one(&connected).await.unwrap();
    assert_eq!(mismatches, 0);
    assert_eq!(
        database::seed(&connected).await.unwrap(),
        SeedSummary {
            files_skipped: 46,
            ..Default::default()
        }
    );
    connected.close().await;
    fixture.close().await;
}

#[tokio::test]
async fn existing_retired_rows_are_preserved_while_missing_rows_are_created() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    let parent: i64 = sqlx::query_scalar("INSERT INTO provinces(kemendagri_code,name,boundary,boundary_source,is_active,deleted_at) VALUES ('11','Preserved province',ST_GeomFromText('MULTIPOLYGON(((100 0,101 0,101 1,100 0)))',4326),'fixture',false,NOW()) RETURNING id").fetch_one(&fixture.pool).await.unwrap();
    sqlx::query("INSERT INTO cities(province_id,kemendagri_code,kind,name,boundary,boundary_source,is_active,deleted_at) VALUES ($1,'11.01','city','Preserved city',ST_GeomFromText('MULTIPOLYGON(((100 0,101 0,101 1,100 0)))',4326),'fixture',false,NOW())").bind(parent).execute(&fixture.pool).await.unwrap();
    let result = database::seed(&fixture.pool).await.unwrap();
    assert_eq!((result.rows_inserted, result.rows_skipped), (553, 2));
    let preserved: (String, bool, bool, String) = sqlx::query_as(
        "SELECT name,is_active,deleted_at IS NOT NULL,boundary_source FROM provinces WHERE id=$1",
    )
    .bind(parent)
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(
        preserved,
        ("Preserved province".into(), false, true, "fixture".into())
    );
    let city: (String, String, bool, bool) = sqlx::query_as("SELECT name,kind::text,is_active,deleted_at IS NOT NULL FROM cities WHERE kemendagri_code='11.01'").fetch_one(&fixture.pool).await.unwrap();
    assert_eq!(city, ("Preserved city".into(), "city".into(), false, true));
    assert_eq!(fixture.count("seed_history").await, 46);
    fixture.close().await;
}

#[tokio::test]
async fn conflicting_checksum_or_filename_stops_seeding() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    database::provinces::seed(&fixture.pool).await.unwrap();
    sqlx::query("UPDATE seed_history SET checksum_sha256=repeat('0',64) WHERE version=1")
        .execute(&fixture.pool)
        .await
        .unwrap();
    let error = database::provinces::seed(&fixture.pool).await.unwrap_err();
    assert_eq!(error.operation, "check seed history");
    assert_eq!(error.version, Some(1));
    sqlx::query(
        "UPDATE seed_history SET checksum_sha256=$1,filename='9999_other.json' WHERE version=1",
    )
    .bind(master_database::seeds::provinces::FILES[0].checksum_sha256)
    .execute(&fixture.pool)
    .await
    .unwrap();
    assert!(database::provinces::seed(&fixture.pool).await.is_err());
    assert_eq!(fixture.count("provinces").await, 38);
    assert_eq!(fixture.count("seed_history").await, 7);
    fixture.close().await;
}

#[tokio::test]
async fn missing_parent_fails_without_partial_city_data_or_history() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    let error = database::cities::seed(&fixture.pool).await.unwrap_err();
    assert_eq!(error.operation, "find city parent province");
    assert_eq!(error.version, Some(8));
    assert_eq!(fixture.count("cities").await, 0);
    assert_eq!(fixture.count("seed_history").await, 0);
    fixture.close().await;
}

#[tokio::test]
async fn failed_file_rolls_back_and_retry_resumes_after_prior_commits() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    sqlx::raw_sql("CREATE FUNCTION reject_province() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.kemendagri_code='32' THEN RAISE EXCEPTION 'fixture failure'; END IF; RETURN NEW; END $$; CREATE TRIGGER reject_province BEFORE INSERT ON provinces FOR EACH ROW EXECUTE FUNCTION reject_province();").execute(&fixture.pool).await.unwrap();
    let error = database::seed(&fixture.pool).await.unwrap_err();
    assert_eq!(error.version, Some(2));
    assert_eq!(error.code.as_deref(), Some("32"));
    assert_eq!(fixture.count("provinces").await, 10);
    assert_eq!(fixture.count("seed_history").await, 1);
    let jawa_rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM provinces WHERE kemendagri_code IN ('31','32')")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(jawa_rows, 0);
    sqlx::raw_sql("DROP TRIGGER reject_province ON provinces; DROP FUNCTION reject_province();")
        .execute(&fixture.pool)
        .await
        .unwrap();
    let retry = database::seed(&fixture.pool).await.unwrap();
    assert_eq!(
        retry,
        SeedSummary {
            files_applied: 45,
            files_skipped: 1,
            rows_inserted: 545,
            rows_skipped: 0
        }
    );
    fixture.close().await;
}

#[tokio::test]
async fn history_write_failure_rolls_back_the_seed_data() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    sqlx::raw_sql("CREATE FUNCTION reject_history() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fixture history failure'; END $$; CREATE TRIGGER reject_history BEFORE INSERT ON seed_history FOR EACH ROW EXECUTE FUNCTION reject_history();").execute(&fixture.pool).await.unwrap();
    let error = database::provinces::seed(&fixture.pool).await.unwrap_err();
    assert_eq!(error.operation, "record seed history");
    assert_eq!(fixture.count("provinces").await, 0);
    assert_eq!(fixture.count("seed_history").await, 0);
    fixture.close().await;
}

#[tokio::test]
async fn independent_runners_do_not_duplicate_records_or_history() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    let second = database::connection::connect(&fixture.env).await.unwrap();
    let (first, other) = tokio::join!(database::seed(&fixture.pool), database::seed(&second));
    let first = first.unwrap();
    let other = other.unwrap();
    assert_eq!(first.files_applied + other.files_applied, 46);
    assert_eq!(first.files_skipped + other.files_skipped, 46);
    assert_eq!(first.rows_inserted + other.rows_inserted, 555);
    assert_eq!(fixture.count("seed_history").await, 46);
    assert_eq!(fixture.count("provinces").await, 38);
    assert_eq!(fixture.count("cities").await, 514);
    second.close().await;
    fixture.close().await;
}

#[tokio::test]
async fn connection_reports_missing_schema_without_creating_it() {
    let Some(fixture) = Fixture::new(false).await else {
        return;
    };
    let error = database::connection::connect(&fixture.env)
        .await
        .unwrap_err();
    let message = error.to_string();
    assert!(message.contains("apply schema migrations first"));
    assert!(message.contains("postgis"));
    assert!(message.contains("seed_history"));
    let postgis: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT FROM pg_extension WHERE extname='postgis')")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert!(!postgis);
    fixture.close().await;
}

#[tokio::test]
async fn polling_methods_preserve_existing_rows_and_reject_history_conflicts() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    sqlx::query("INSERT INTO polling_methods (id, code, name, description, is_active, deleted_at) VALUES (100, 'rating', 'Existing rating', 'Preserved description', false, NOW())")
        .execute(&fixture.pool).await.unwrap();
    let seeded = database::polling_methods::seed(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(
        seeded,
        SeedSummary {
            files_applied: 1,
            rows_inserted: 2,
            rows_skipped: 1,
            files_skipped: 0
        }
    );
    let preserved: (i64, String, bool, bool) = sqlx::query_as("SELECT id, name, is_active, deleted_at IS NOT NULL FROM polling_methods WHERE code='rating'")
        .fetch_one(&fixture.pool).await.unwrap();
    assert_eq!(preserved, (100, "Existing rating".into(), false, true));
    assert_eq!(fixture.count("polling_methods").await, 3);
    assert_eq!(
        database::polling_methods::seed(&fixture.pool)
            .await
            .unwrap()
            .files_skipped,
        1
    );
    sqlx::query("UPDATE seed_history SET checksum_sha256=repeat('0',64) WHERE version=46")
        .execute(&fixture.pool)
        .await
        .unwrap();
    let error = database::polling_methods::seed(&fixture.pool)
        .await
        .unwrap_err();
    assert_eq!(error.version, Some(46));
    assert_eq!(error.operation, "check seed history");
    fixture.close().await;
}

#[tokio::test]
async fn polling_method_seed_rolls_back_rows_and_history_on_failure() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    // Test-only trigger simulates an arbitrary write failure in the middle of a file.
    sqlx::raw_sql("CREATE FUNCTION reject_method() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF NEW.code='rating' THEN RAISE EXCEPTION 'fixture failure'; END IF; RETURN NEW; END $$; CREATE TRIGGER reject_method BEFORE INSERT ON polling_methods FOR EACH ROW EXECUTE FUNCTION reject_method();")
        .execute(&fixture.pool).await.unwrap();
    let error = database::polling_methods::seed(&fixture.pool)
        .await
        .unwrap_err();
    assert_eq!(error.code.as_deref(), Some("rating"));
    assert_eq!(fixture.count("polling_methods").await, 0);
    assert_eq!(fixture.count("seed_history").await, 0);
    sqlx::raw_sql("DROP TRIGGER reject_method ON polling_methods; DROP FUNCTION reject_method(); CREATE FUNCTION reject_method_history() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE EXCEPTION 'fixture history failure'; END $$; CREATE TRIGGER reject_method_history BEFORE INSERT ON seed_history FOR EACH ROW EXECUTE FUNCTION reject_method_history();")
        .execute(&fixture.pool).await.unwrap();
    let error = database::polling_methods::seed(&fixture.pool)
        .await
        .unwrap_err();
    assert_eq!(error.operation, "record seed history");
    assert_eq!(fixture.count("polling_methods").await, 0);
    assert_eq!(fixture.count("seed_history").await, 0);
    sqlx::raw_sql("DROP TRIGGER reject_method_history ON seed_history; DROP FUNCTION reject_method_history();")
        .execute(&fixture.pool).await.unwrap();
    let retry = database::polling_methods::seed(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(retry.rows_inserted, 3);
    assert_eq!(fixture.count("seed_history").await, 1);
    fixture.close().await;
}

#[tokio::test]
async fn topics_reference_methods_and_store_native_arrays_without_business_triggers() {
    let Some(fixture) = Fixture::new(true).await else {
        return;
    };
    database::polling_methods::seed(&fixture.pool)
        .await
        .unwrap();
    let selection: i64 =
        sqlx::query_scalar("SELECT id FROM polling_methods WHERE code='selection'")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    let id: i64 = sqlx::query_scalar("INSERT INTO topics (slug, name, polling_method_id, selection_options) VALUES ('selection-topic','Selection topic',$1,$2) RETURNING id")
        .bind(selection).bind(vec!["Yes", "No"]).fetch_one(&fixture.pool).await.unwrap();
    let options: Vec<String> =
        sqlx::query_scalar("SELECT selection_options FROM topics WHERE id=$1")
            .bind(id)
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(options, ["Yes", "No"]);
    let error = sqlx::query("INSERT INTO topics (slug, name, polling_method_id) VALUES ('missing','Missing',9223372036854775807)")
        .execute(&fixture.pool).await.unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23503")
    );
    let error = sqlx::query("INSERT INTO topics (slug, name) VALUES ('no-method','No method')")
        .execute(&fixture.pool)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23502")
    );
    let error = sqlx::query("DELETE FROM polling_methods WHERE id=$1")
        .bind(selection)
        .execute(&fixture.pool)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23503")
    );
    let error = sqlx::query("UPDATE polling_methods SET id=id+1000 WHERE id=$1")
        .bind(selection)
        .execute(&fixture.pool)
        .await
        .unwrap_err();
    assert_eq!(
        error.as_database_error().unwrap().code().as_deref(),
        Some("23503")
    );
    for sql in [
        "UPDATE topics SET selection_options=NULL WHERE slug='selection-topic'",
        "UPDATE topics SET selection_options=ARRAY[['A','B'],['C','D']] WHERE slug='selection-topic'",
        "UPDATE topics SET selection_options='[0:1]={A,B}'::text[] WHERE slug='selection-topic'",
    ] {
        assert!(sqlx::query(sql).execute(&fixture.pool).await.is_err());
    }
    // Method-dependent rules intentionally belong to the future master_service.
    sqlx::query("UPDATE topics SET selection_options='{}' WHERE id=$1")
        .bind(id)
        .execute(&fixture.pool)
        .await
        .unwrap();
    let defaults: Vec<String> =
        sqlx::query_scalar("SELECT selection_options FROM topics WHERE id=$1")
            .bind(id)
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert!(defaults.is_empty());
    let triggers: i64 = sqlx::query_scalar("SELECT count(*) FROM pg_trigger WHERE tgrelid IN ('topics'::regclass, 'polling_methods'::regclass) AND NOT tgisinternal")
        .fetch_one(&fixture.pool).await.unwrap();
    assert_eq!(triggers, 0);
    for index in [
        "uq_polling_methods_code",
        "idx_polling_methods_is_active",
        "idx_polling_methods_deleted_at",
        "idx_topics_polling_method_id",
    ] {
        assert!(
            sqlx::query_scalar::<_, bool>("SELECT to_regclass($1) IS NOT NULL")
                .bind(index)
                .fetch_one(&fixture.pool)
                .await
                .unwrap()
        );
    }
    let mut paths: Vec<_> =
        std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.to_string_lossy().ends_with(".down.sql"))
            .collect();
    paths.sort();
    for path in paths.into_iter().rev() {
        sqlx::raw_sql(sqlx::AssertSqlSafe(std::fs::read_to_string(path).unwrap()))
            .execute(&fixture.pool)
            .await
            .unwrap();
    }
    let removed: bool = sqlx::query_scalar(
        "SELECT to_regclass('topics') IS NULL AND to_regclass('polling_methods') IS NULL",
    )
    .fetch_one(&fixture.pool)
    .await
    .unwrap();
    assert!(removed);
    fixture.close().await;
}
