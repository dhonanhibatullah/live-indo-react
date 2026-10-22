use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    master_service::composition::launch().await
}
