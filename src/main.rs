use flashgrep::init_logging;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    init_logging();

    match flashgrep::cli::run().await {
        Ok(outcome) => ExitCode::from(outcome.exit_code()),
        Err(e) => {
            eprintln!("Error: {}", e);
            let code = e.exit_code().clamp(1, 255) as u8;
            ExitCode::from(code)
        }
    }
}
