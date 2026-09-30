//! Host ports and runtime injection.
//!
//! Every external dependency is a trait parameter on `Runtime`, so the core
//! can be driven by deterministic fakes. Git, Claude, Codex, OpenCode, and doob
//! are reached only through `ProcessRunner`; nothing in the core shells out
//! directly.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::ExitStatus;

use crate::capability::RustOperation;
use crate::compat::CompatibilityProfile;

/// A fully specified child process to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub(crate) program: OsString,
    pub(crate) arguments: Vec<OsString>,
    pub(crate) working_directory: Option<PathBuf>,
    pub(crate) environment: BTreeMap<OsString, OsString>,
    pub(crate) stdin: Vec<u8>,
}

impl CommandSpec {
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            arguments: Vec::new(),
            working_directory: None,
            environment: BTreeMap::new(),
            stdin: Vec::new(),
        }
    }

    pub fn arg(mut self, argument: impl Into<OsString>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    pub fn args<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.arguments.extend(arguments.into_iter().map(Into::into));
        self
    }

    pub fn cwd(mut self, directory: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(directory.into());
        self
    }

    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    pub fn stdin(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.stdin = bytes.into();
        self
    }

    pub fn program(&self) -> &OsStr {
        &self.program
    }

    pub fn arguments(&self) -> &[OsString] {
        &self.arguments
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.status.success()
    }
}

/// A resolved operation, ready to execute without redoing compatibility or
/// alias resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationInvocation {
    pub(crate) operation: RustOperation,
    pub(crate) profile: CompatibilityProfile,
    pub(crate) arguments: Vec<OsString>,
    pub(crate) working_directory: PathBuf,
    pub(crate) stdin: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperationResult {
    pub(crate) exit_code: u8,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
}

/// The injected world. Generic over every port so tests substitute fakes.
///
/// The ports are read by later plan tasks; nothing in this task dispatches
/// through them yet.
#[derive(Debug)]
#[expect(
    dead_code,
    reason = "ports are consumed by plan_generation, apply_generation, and dispatch_hook"
)]
pub struct Runtime<F, P, C, E, S, H> {
    pub(crate) file_system: F,
    pub(crate) process_runner: P,
    pub(crate) clock: C,
    pub(crate) environment: E,
    pub(crate) handoff_store: S,
    pub(crate) hook_handler: H,
}

impl<F, P, C, E, S, H> Runtime<F, P, C, E, S, H> {
    pub fn new(
        file_system: F,
        process_runner: P,
        clock: C,
        environment: E,
        handoff_store: S,
        hook_handler: H,
    ) -> Self {
        Self {
            file_system,
            process_runner,
            clock,
            environment,
            handoff_store,
            hook_handler,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn command_spec_builds_argv_in_order() {
        let spec = CommandSpec::new("git")
            .args(["status", "--short"])
            .arg("--branch")
            .cwd("/tmp")
            .env("GIT_OPTIONAL_LOCKS", "0");
        assert_eq!(spec.program(), OsStr::new("git"));
        assert_eq!(spec.arguments().len(), 3);
        assert_eq!(spec.arguments()[0], OsString::from("status"));
        assert_eq!(spec.arguments()[2], OsString::from("--branch"));
        assert_eq!(spec.working_directory.as_deref(), Some(Path::new("/tmp")));
        assert_eq!(
            spec.environment.get(&OsString::from("GIT_OPTIONAL_LOCKS")),
            Some(&OsString::from("0"))
        );
    }

    #[test]
    fn runtime_holds_every_injected_port() {
        let runtime = Runtime::new(1_u8, 2_u8, 3_u8, 4_u8, 5_u8, 6_u8);
        assert_eq!(runtime.file_system, 1);
        assert_eq!(runtime.hook_handler, 6);
    }
}
