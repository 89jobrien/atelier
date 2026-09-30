//! Hook request, response, and decision values.
//!
//! A decision is expressed once and adapted per harness. An adapter rejects a
//! decision its harness cannot represent rather than silently weakening it, so
//! a `Block` never degrades into a warning on a harness that lacks the concept.

use std::path::PathBuf;

use crate::capability::LifecycleEventId;
use crate::harness::HarnessId;

/// A normalized hook invocation, independent of the harness wire format.
#[derive(Debug, Clone, PartialEq)]
pub struct HookRequest {
    pub(crate) harness: HarnessId,
    pub(crate) event: LifecycleEventId,
    pub(crate) working_directory: PathBuf,
    pub(crate) session_id: Option<String>,
    pub(crate) payload: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HookResponse {
    pub(crate) decision: HookDecision,
    pub(crate) additional_context: Option<String>,
    pub(crate) diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HookDecision {
    Continue,
    Allow { reason: Option<String> },
    Block { reason: String },
    ReplaceInput { input: serde_json::Value },
    AddContext { text: String },
}

/// The decision kinds an action may declare as outcomes. Separate from
/// `HookDecision` because it carries no payload and is used as a capability
/// constraint in the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HookDecisionKind {
    Continue,
    Allow,
    Block,
    ReplaceInput,
    AddContext,
}

impl HookDecisionKind {
    pub const ALL: [Self; 5] = [
        Self::Continue,
        Self::Allow,
        Self::Block,
        Self::ReplaceInput,
        Self::AddContext,
    ];

    /// The kind of a concrete decision, ignoring any payload it carries.
    pub fn of(decision: &HookDecision) -> Self {
        match decision {
            HookDecision::Continue => Self::Continue,
            HookDecision::Allow { .. } => Self::Allow,
            HookDecision::Block { .. } => Self::Block,
            HookDecision::ReplaceInput { .. } => Self::ReplaceInput,
            HookDecision::AddContext { .. } => Self::AddContext,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_decision_maps_to_a_kind() {
        let cases = [
            (HookDecision::Continue, HookDecisionKind::Continue),
            (
                HookDecision::Allow { reason: None },
                HookDecisionKind::Allow,
            ),
            (
                HookDecision::Block {
                    reason: "nope".into(),
                },
                HookDecisionKind::Block,
            ),
            (
                HookDecision::ReplaceInput {
                    input: serde_json::Value::Null,
                },
                HookDecisionKind::ReplaceInput,
            ),
            (
                HookDecision::AddContext { text: "x".into() },
                HookDecisionKind::AddContext,
            ),
        ];
        for (decision, expected) in cases {
            assert_eq!(HookDecisionKind::of(&decision), expected);
        }
    }

    #[test]
    fn kinds_are_five_and_distinct() {
        assert_eq!(HookDecisionKind::ALL.len(), 5);
        let mut kinds = HookDecisionKind::ALL.to_vec();
        kinds.sort_by_key(|k| format!("{k:?}"));
        kinds.dedup();
        assert_eq!(kinds.len(), 5);
    }
}
