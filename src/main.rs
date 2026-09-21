//! Forwards process arguments to Atelier's reusable CLI runner.

use std::process::ExitCode;

fn main() -> ExitCode {
    atelier::try_run(std::env::args_os())
}
