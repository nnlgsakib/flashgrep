use flashgrep::init_logging;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    init_logging();

    match flashgrep::cli::run().await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::from(1)
        }
    }
}
