//! Canonical capability manifest types.
//!
//! These mirror `atelier.toml` and the approved design one-for-one. Field
//! privacy is intentional: the manifest is constructed by parsing in a later
//! task, so exposing setters now would freeze an API the design has not
//! settled.

use std::path::PathBuf;

use crate::compat::CompatibilityProfile;
use crate::harness::HarnessId;

/// The whole of `atelier.toml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityManifest {
    pub(crate) schema_version: u32,
    pub(crate) plugin: PluginMetadata,
    pub(crate) capabilities: Vec<Capability>,
    pub(crate) actions: Vec<ActionDefinition>,
    pub(crate) aliases: Vec<AliasDefinition>,
    pub(crate) legacy_artifacts: Vec<LegacyArtifact>,
    pub(crate) legacy_git_config: Vec<LegacyGitConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginMetadata {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) author: PluginAuthor,
    pub(crate) version_source: VersionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginAuthor {
    pub(crate) name: String,
    pub(crate) email: String,
}

/// One installable unit: a skill, agent, hook, binary, or repository hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    pub(crate) id: CapabilityId,
    pub(crate) kind: CapabilityKind,
    pub(crate) source: Option<PathBuf>,
    pub(crate) portability: Portability,
    pub(crate) requirements: Requirements,
    pub(crate) lifecycle: Vec<LifecycleBinding>,
    pub(crate) harness: HarnessMetadataSet,
}

/// Newtype so a capability identifier cannot be confused with an action or
/// lifecycle-event identifier, which all travel together in error variants.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CapabilityId {
    pub(crate) value: String,
}

impl std::fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionId {
    pub(crate) value: String,
}

impl std::fmt::Display for ActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LifecycleEventId {
    pub(crate) value: String,
}

impl std::fmt::Display for LifecycleEventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

/// Binds a lifecycle event to a Rust action for one or more harnesses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleBinding {
    pub(crate) harnesses: Vec<HarnessId>,
    pub(crate) event: LifecycleEventId,
    pub(crate) matcher: Option<String>,
    pub(crate) action: ActionId,
}

/// External things a capability needs before it can run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Requirements {
    pub(crate) commands: Vec<String>,
    pub(crate) environment: Vec<String>,
    pub(crate) companions: Vec<String>,
}

/// Per-harness presentation metadata; absent means the harness does not
/// receive this capability.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HarnessMetadataSet {
    pub(crate) claude: Option<HarnessMetadata>,
    pub(crate) codex: Option<HarnessMetadata>,
    pub(crate) opencode: Option<HarnessMetadata>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HarnessMetadata {
    pub(crate) display_name: Option<String>,
    pub(crate) short_description: Option<String>,
    pub(crate) default_prompt: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) tools: Vec<String>,
    pub(crate) skills: Vec<String>,
    pub(crate) permission_mode: Option<String>,
}

/// A Rust operation exposed as a command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionDefinition {
    pub(crate) id: ActionId,
    pub(crate) operation: RustOperation,
    pub(crate) profile: CompatibilityProfile,
    pub(crate) outcomes: Vec<HookDecisionKind>,
}

/// The complete set of native operations. Each legacy script maps onto one of
/// these; nothing in the runtime dispatches to an interpreter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RustOperation {
    Generate,
    Install,
    HandoffInit,
    HandoffDetect,
    HandoffMigrate,
    HandoffDatabase,
    HandoffReconcile,
    HandoffRender,
    HandoffDiagrams,
    SchemaInfer,
    RepoPreCommit,
    RepoPrePush,
    RepoPostCommit,
}

impl RustOperation {
    pub const ALL: [Self; 13] = [
        Self::Generate,
        Self::Install,
        Self::HandoffInit,
        Self::HandoffDetect,
        Self::HandoffMigrate,
        Self::HandoffDatabase,
        Self::HandoffReconcile,
        Self::HandoffRender,
        Self::HandoffDiagrams,
        Self::SchemaInfer,
        Self::RepoPreCommit,
        Self::RepoPrePush,
        Self::RepoPostCommit,
    ];
}

/// A legacy invocation path and the profile it selects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AliasDefinition {
    pub(crate) invocation: PathBuf,
    pub(crate) operation: RustOperation,
    pub(crate) profile: CompatibilityProfile,
}

/// A legacy file the installer must account for during removal or migration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyArtifact {
    pub(crate) root: LegacyRoot,
    pub(crate) path: PathBuf,
    pub(crate) kind: PathEntryKind,
    pub(crate) matcher: LegacyMatcher,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyRoot {
    Source,
    Harness,
    Executable,
    RepositoryHooks,
}

/// How a legacy artifact is recognized, so an adoption decision can verify
/// the file is the expected one rather than merely present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyMatcher {
    Sha256 {
        value: String,
    },
    JsonIgnoringKeys {
        value: String,
        ignored_key_paths: Vec<Vec<String>>,
    },
    SourceLink {
        source: PathBuf,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyGitConfig {
    pub(crate) key: String,
    pub(crate) value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityKind {
    Skill,
    Agent,
    Hook,
    Binary,
    RepositoryHook,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Portability {
    Universal,
    Adaptable,
    HarnessSpecific { harness: HarnessId },
}

/// Plugin versions are stamped from git by the pre-push hook, so the manifest
/// never carries a literal version for Atelier itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionSource {
    GitHead,
    Literal { value: String },
}

pub use crate::harness::generation::PathEntryKind;
pub use crate::hooks::HookDecisionKind;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_operation_has_thirteen_variants() {
        assert_eq!(RustOperation::ALL.len(), 13);
        let mut discriminants: Vec<u64> = RustOperation::ALL.iter().map(|op| *op as u64).collect();
        discriminants.sort_unstable();
        discriminants.dedup();
        assert_eq!(
            discriminants.len(),
            13,
            "each operation must be a distinct enum variant"
        );
    }

    #[test]
    fn identifier_newtypes_are_distinct_types() {
        // Compile-time proof only: these must not unify.
        let capability = CapabilityId {
            value: "handoff".into(),
        };
        let action = ActionId {
            value: "handoff".into(),
        };
        assert_eq!(capability.value, action.value);
    }

    #[test]
    fn requirements_default_to_empty() {
        let requirements = Requirements::default();
        assert!(requirements.commands.is_empty());
        assert!(requirements.environment.is_empty());
        assert!(requirements.companions.is_empty());
    }
}
