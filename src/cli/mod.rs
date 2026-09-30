//! Defines command-line parsing and dispatch for the Atelier runtime.

use std::{ffi::OsString, process::ExitCode};

use clap::{Parser, Subcommand};

use crate::AtelierError;

#[derive(Debug, Parser)]
#[command(name = "atelier", version)]
#[command(about = "Generate and run Atelier harness integrations")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Validate,
    Generate,
    Install,
    Hook,
    Handoff {
        #[command(subcommand)]
        command: HandoffCommand,
    },
    Schema {
        #[command(subcommand)]
        command: SchemaCommand,
    },
    RepoHook {
        #[command(subcommand)]
        command: RepoHookCommand,
    },
}

#[derive(Debug, Subcommand)]
enum HandoffCommand {
    Init,
    Detect,
    Migrate,
    Db,
    Reconcile,
    Render,
    Diagrams,
}

#[derive(Debug, Subcommand)]
enum SchemaCommand {
    Infer,
}

#[derive(Debug, Subcommand)]
enum RepoHookCommand {
    PreCommit,
    PrePush,
    PostCommit,
}

/// Parses the supplied command-line arguments and returns the resulting process exit code.
///
/// Errors propagate to `main`, which prints them to stderr; the library never
/// writes to the process streams itself so it stays testable in-process.
pub fn try_run<I, T>(arguments: I) -> Result<ExitCode, AtelierError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let _cli = Cli::parse_from(arguments);
    Ok(ExitCode::SUCCESS)
}
