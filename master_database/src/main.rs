use std::{
    io::{self, Write},
    path::Path,
    process::ExitCode,
};

use master_database::{config, database};

#[tokio::main]
async fn main() -> ExitCode {
    let app = config::app::load();
    let _ = writeln!(
        io::stdout().lock(),
        "[INFO] {} {} starting",
        app.name,
        app.version
    );

    let env = match config::env::load() {
        Ok(env) => env,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "[ERROR] load configuration: {error}");
            return ExitCode::FAILURE;
        }
    };

    let pool = match database::connection::connect_for_migrations(&env).await {
        Ok(pool) => pool,
        // The connection utility logs safe diagnostics before returning an error.
        Err(_) => return ExitCode::FAILURE,
    };

    let result = async {
        database::migrations::run(&pool, Path::new("migrations")).await?;
        database::connection::verify_schema(&pool).await?;
        database::seed(&pool).await
    }
    .await;
    pool.close().await;
    let _ = writeln!(
        io::stdout().lock(),
        "[INFO] database connection pool closed"
    );

    match result {
        Ok(_) => {
            let _ = writeln!(io::stdout().lock(), "[INFO] {} completed", app.name);
            ExitCode::SUCCESS
        }
        // Database operations log failures; shutdown still runs before exiting.
        Err(_) => ExitCode::FAILURE,
    }
}
