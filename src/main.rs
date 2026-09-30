//! Forwards process arguments to Atelier's reusable CLI runner.

use std::process::ExitCode;

fn main() -> ExitCode {
    match atelier::try_run(std::env::args_os()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("atelier: {error}");
            let mut source = std::error::Error::source(&error);
            while let Some(cause) = source {
                eprintln!("  caused by: {cause}");
                source = cause.source();
            }
            ExitCode::FAILURE
        }
    }
}
