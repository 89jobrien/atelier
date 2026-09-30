//! Atelier: generate and run harness integrations.
//!
//! The crate is organised as a hexagonal core: every external dependency is a
//! trait parameter on [`host::Runtime`], so the core can be driven by
//! deterministic fakes. Git, Claude, Codex, OpenCode, and doob are reached only
//! through `ProcessRunner`; Python, Nushell, jq, and shell interpreters are not
//! runtime dependencies.

pub mod capability;
pub mod cli;
pub mod compat;
pub mod handoff;
pub mod harness;
pub mod hooks;
pub mod host;

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitStatus;

use thiserror::Error;

pub use capability::{ActionId, CapabilityId, LifecycleEventId, RustOperation};
pub use cli::try_run;
pub use compat::{
    CompatibilityProfile, CompatibilitySelection, CompatibilitySource, resolve_compatibility,
};
pub use harness::HarnessId;
pub use hooks::HookDecisionKind;

/// The single library error type. Variants keep their sources so callers can
/// walk the chain rather than matching on formatted strings.
#[derive(Debug, Error)]
pub enum AtelierError {
    #[error("io error{}", path_suffix(.path))]
    Io {
        path: Option<PathBuf>,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid manifest: {message}")]
    InvalidManifest { message: String },
    #[error("toml error in {}", .path.display())]
    Toml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml error: {0}")]
    Yaml(#[source] serde_yaml_ng::Error),
    #[error("process {} failed with {}: {}", .program.to_string_lossy(), .status, .stderr)]
    Process {
        program: OsString,
        status: ExitStatus,
        stderr: String,
    },
    #[error("capability {} is unsupported on {harness}: {reason}", .capability)]
    UnsupportedCapability {
        harness: HarnessId,
        capability: CapabilityId,
        reason: String,
    },
    #[error("ownership conflict at {}", .path.display())]
    OwnershipConflict { path: PathBuf },
    #[error("hook binding {action} for {harness}/{event}: {message}")]
    HookBinding {
        harness: HarnessId,
        event: LifecycleEventId,
        action: ActionId,
        message: String,
    },
    #[error("recovery failed for journal {}: {message}", .journal.display())]
    Recovery { journal: PathBuf, message: String },
    #[error("store error: {message}")]
    Store { message: String },
    #[error("compatibility profile {profile} failed: {message}")]
    Compatibility {
        profile: CompatibilityProfile,
        message: String,
    },
}

fn path_suffix(path: &Option<PathBuf>) -> String {
    match path {
        Some(p) => format!(" at {}", p.display()),
        None => String::new(),
    }
}

impl AtelierError {
    /// Build an I/O error that remembers the path it happened on.
    pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: Some(path.into()),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn io_error_mentions_its_path() {
        let error = AtelierError::io(
            "/tmp/x",
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        );
        assert!(error.to_string().contains("/tmp/x"));
    }

    #[test]
    fn bare_io_error_has_no_path_suffix() {
        let error = AtelierError::Io {
            path: None,
            source: std::io::Error::other("boom"),
        };
        assert_eq!(error.to_string(), "io error");
    }

    #[test]
    fn json_converts_and_preserves_source() {
        let source = serde_json::from_str::<u8>("not json").unwrap_err();
        let error = AtelierError::from(source);
        assert!(matches!(error, AtelierError::Json(_)));
    }
}
