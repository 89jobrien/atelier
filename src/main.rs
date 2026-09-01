use std::process::ExitCode;

fn main() -> ExitCode {
    atelier::try_run(std::env::args_os())
}
