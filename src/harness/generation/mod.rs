//! Generation planning types.
//!
//! A plan is the complete, reviewable description of what an install will do.
//! Planning never touches the filesystem; only `apply_generation` does.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::capability::CapabilityId;
use crate::harness::HarnessId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationPlan {
    pub(crate) harness: HarnessId,
    pub(crate) scope: InstallScope,
    pub(crate) state_root: PathBuf,
    pub(crate) changes: Vec<FileChange>,
    pub(crate) parity: ParityReport,
    pub(crate) receipt: OwnershipReceipt,
}

/// Everything planning needs that is not in the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationContext {
    pub(crate) scope: InstallScope,
    pub(crate) source_root: PathBuf,
    pub(crate) target_root: PathBuf,
    pub(crate) state_root: PathBuf,
    pub(crate) executable_root: PathBuf,
    pub(crate) repository_hooks_root: PathBuf,
    pub(crate) sources: BTreeMap<PathBuf, Vec<u8>>,
    /// Observed external state, such as `core.hooksPath`. Values are read
    /// during planning; writing one requires adoption.
    pub(crate) external_state: BTreeMap<String, Option<String>>,
    pub(crate) git_head: String,
    pub(crate) previous_receipt: Option<OwnershipReceipt>,
    pub(crate) migration_mode: MigrationMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationMode {
    Strict,
    AdoptMatching,
    ReplaceKnownLegacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstallScope {
    User,
    Project,
}

impl InstallScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Project => "project",
        }
    }
}

/// A single planned filesystem or Git-config mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChange {
    Create {
        path: PathBuf,
        content: Vec<u8>,
    },
    Replace {
        path: PathBuf,
        content: Vec<u8>,
    },
    RemoveManaged {
        path: PathBuf,
    },
    InstallExecutable {
        source: PathBuf,
        target: PathBuf,
    },
    Link {
        source: PathBuf,
        target: PathBuf,
    },
    MergeDocument {
        path: PathBuf,
        format: DocumentFormat,
        key_path: Vec<String>,
        content: Vec<u8>,
    },
    RemoveManagedDocumentKey {
        path: PathBuf,
        format: DocumentFormat,
        key_path: Vec<String>,
    },
    SetGitConfig {
        repository: PathBuf,
        key: String,
        previous_value: Option<String>,
        value: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentFormat {
    Json,
    Toml,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParityReport {
    pub(crate) entries: Vec<ParityEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityEntry {
    pub(crate) capability: CapabilityId,
    pub(crate) status: ParityStatus,
    pub(crate) detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParityStatus {
    Native,
    Adapted,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathEntry {
    pub(crate) path: PathBuf,
    pub(crate) kind: PathEntryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathEntryKind {
    File,
    Directory,
    Symlink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    pub(crate) kind: PathEntryKind,
    pub(crate) executable: bool,
    pub(crate) length: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyMode {
    DryRun,
    Write,
}

// Ownership types live in their own module because receipts are the commit
// point of every install.
pub use super::ownership::{OwnershipReceipt, OwnershipRecord};
