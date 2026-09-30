//! HANDOFF document model.
//!
//! Every model carries `extra_fields` so an unrecognized key in a user's
//! HANDOFF file survives a read-modify-write round trip. Dropping unknown
//! keys would silently delete hand-written content on the next commit.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_yaml_ng::Value;

/// The committed source of truth: `.ctx/HANDOFF.<name>.<base>.yaml`.
#[derive(Debug, Clone, PartialEq)]
pub struct HandoffDocument {
    pub(crate) project: String,
    pub(crate) id: Option<String>,
    pub(crate) updated: Option<String>,
    pub(crate) items: Vec<HandoffItem>,
    pub(crate) log: Vec<HandoffLogEntry>,
    pub(crate) extra_fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HandoffItem {
    pub(crate) id: String,
    pub(crate) name: Option<String>,
    pub(crate) priority: Option<String>,
    pub(crate) status: ItemStatus,
    pub(crate) title: Option<String>,
    pub(crate) description: Option<String>,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) depends_on: Vec<String>,
    pub(crate) completed: Option<String>,
    pub(crate) updated: Option<String>,
    pub(crate) extra: Vec<HandoffExtraEntry>,
    pub(crate) extra_fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HandoffExtraEntry {
    pub(crate) date: Option<String>,
    pub(crate) kind: Option<String>,
    pub(crate) field: Option<String>,
    pub(crate) value: Option<Value>,
    pub(crate) reviewed: Option<String>,
    pub(crate) note: Option<String>,
    pub(crate) extra_fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HandoffLogEntry {
    pub(crate) date: Option<String>,
    pub(crate) session: Option<u64>,
    pub(crate) claude_session_id: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) commits: Vec<HandoffCommit>,
    pub(crate) extra_fields: BTreeMap<String, Value>,
}

/// A commit reference that tolerates both the object form and the bare-sha
/// form found in older files.
#[derive(Debug, Clone, PartialEq)]
pub enum HandoffCommit {
    Object {
        sha: String,
        branch: Option<String>,
        extra_fields: BTreeMap<String, Value>,
    },
    LegacyScalar {
        value: String,
    },
}

/// The uncommitted snapshot: `.ctx/HANDOFF.<name>.<base>.state.yaml`.
#[derive(Debug, Clone, PartialEq)]
pub struct HandoffState {
    pub(crate) updated: Option<String>,
    pub(crate) branch: Option<String>,
    pub(crate) build: Option<String>,
    pub(crate) tests: Option<String>,
    pub(crate) notes: Option<String>,
    pub(crate) touched_files: Vec<PathBuf>,
    pub(crate) extra_fields: BTreeMap<String, Value>,
}

/// Identity of an item across projects, used by the reconciliation bridge.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemKey {
    pub(crate) project: String,
    pub(crate) id: String,
}

/// Deliberately not `Copy`: the `Unknown` variant carries the unrecognized
/// value so it round-trips instead of being coerced to `Open`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemStatus {
    Open,
    Done,
    Parked,
    Blocked,
    PendingValidation,
    Closed,
    /// Preserved verbatim: an unrecognized status must not be coerced into
    /// `Open`, which would change its meaning without telling anyone.
    Unknown {
        value: String,
    },
}

impl ItemStatus {
    pub fn parse(value: &str) -> Self {
        match value {
            "open" => Self::Open,
            "done" => Self::Done,
            "parked" => Self::Parked,
            "blocked" => Self::Blocked,
            "pending_validation" => Self::PendingValidation,
            "closed" => Self::Closed,
            other => Self::Unknown {
                value: other.to_string(),
            },
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Open => "open",
            Self::Done => "done",
            Self::Parked => "parked",
            Self::Blocked => "blocked",
            Self::PendingValidation => "pending_validation",
            Self::Closed => "closed",
            Self::Unknown { value } => value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_statuses_round_trip() {
        for raw in [
            "open",
            "done",
            "parked",
            "blocked",
            "pending_validation",
            "closed",
        ] {
            assert_eq!(ItemStatus::parse(raw).as_str(), raw);
        }
    }

    #[test]
    fn unknown_status_is_preserved_not_coerced() {
        let status = ItemStatus::parse("in-review");
        assert!(matches!(status, ItemStatus::Unknown { .. }));
        assert_eq!(status.as_str(), "in-review");
    }

    #[test]
    fn unknown_status_does_not_equal_open() {
        assert_ne!(ItemStatus::parse("in-review"), ItemStatus::Open);
    }
}
