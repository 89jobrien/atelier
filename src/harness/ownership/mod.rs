//! Ownership receipts and the transaction journal.
//!
//! The receipt is the commit point of an install: a target may be replaced or
//! removed only when the previous receipt names it and the stored hash still
//! matches live content. A user-modified or unmanaged target is a conflict,
//! never a silent overwrite.

use std::path::PathBuf;

use crate::harness::HarnessId;
use crate::harness::generation::InstallScope;

/// The commit record for one harness at one scope, stored as JSON at
/// `<state-root>/receipts/<harness>-<scope>.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipReceipt {
    pub(crate) schema_version: u32,
    pub(crate) harness: HarnessId,
    pub(crate) scope: InstallScope,
    pub(crate) target_root: PathBuf,
    pub(crate) records: Vec<OwnershipRecord>,
}

impl OwnershipReceipt {
    /// `<harness>-<scope>`, matching the receipt filename stem.
    pub fn file_stem(&self) -> String {
        format!("{}-{}", self.harness.as_str(), self.scope.as_str())
    }
}

/// One owned thing and the hash of the content Atelier wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnershipRecord {
    pub(crate) target: ManagedTarget,
    pub(crate) sha256: String,
}

/// Something Atelier owns. A plain path, a single key inside a shared
/// document, or a Git config setting.
///
/// The document-key variant matters because unrelated user edits elsewhere in
/// the same file must not read as an ownership conflict: hashing covers only
/// the owned key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManagedTarget {
    Path {
        path: PathBuf,
    },
    DocumentKey {
        path: PathBuf,
        key_path: Vec<String>,
    },
    GitConfig {
        repository: PathBuf,
        key: String,
        previous_value: Option<String>,
        managed_value: String,
    },
}

/// Recovery state, written before any mutation happens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerationJournal {
    pub(crate) schema_version: u32,
    pub(crate) harness: HarnessId,
    pub(crate) target_root: PathBuf,
    pub(crate) state: JournalState,
    pub(crate) receipt_path: PathBuf,
    /// Exact prior receipt bytes, restored verbatim on rollback.
    pub(crate) previous_receipt: Option<Vec<u8>>,
    /// A live receipt matching this hash means the transaction committed, even
    /// if the journal still reads `Applying`.
    pub(crate) next_receipt_sha256: String,
    pub(crate) completed: Vec<JournalOperation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalState {
    Prepared,
    Applying,
    Committed,
    RollingBack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalOperation {
    pub(crate) target: ManagedTarget,
    pub(crate) backup: Option<PathBuf>,
    pub(crate) staged: Option<PathBuf>,
}

impl JournalOperation {
    /// Transaction IDs are `<harness>-<pid>-<unix-nanos>`, which keeps
    /// concurrent installs of the same harness from colliding on stage and
    /// backup filenames.
    pub fn transaction_id(harness: HarnessId, pid: u32, nanos: u128) -> String {
        format!("{}-{pid}-{nanos}", harness.as_str())
    }
}

/// Re-exported so `ManagedTarget` and `DocumentFormat` stay usable together.
pub use crate::harness::generation::DocumentFormat as DocumentKind;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_id_has_harness_pid_and_nanos() {
        let id = JournalOperation::transaction_id(HarnessId::Claude, 4321, 1_700_000_000_000_000);
        assert_eq!(id, "claude-4321-1700000000000000");
    }

    #[test]
    fn transaction_ids_differ_across_harnesses() {
        let a = JournalOperation::transaction_id(HarnessId::Claude, 1, 2);
        let b = JournalOperation::transaction_id(HarnessId::Codex, 1, 2);
        assert_ne!(a, b);
    }
}
