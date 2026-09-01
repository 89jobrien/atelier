use std::{ffi::OsString, process::ExitCode};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "atelier")]
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

pub fn try_run<I, T>(arguments: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let _cli = Cli::parse_from(arguments);
    ExitCode::SUCCESS
}
