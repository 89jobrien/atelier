//! Compatibility profiles and invocation resolution.
//!
//! `atelier` is a multicall binary. An installed alias carries no marker; it
//! selects behaviour from the path it was invoked through. Resolution is
//! therefore ordered, and the order is part of the contract:
//!
//! 1. explicit `--compat`
//! 2. the full invocation path, matched exactly
//! 3. the executable basename
//! 4. `Canonical`
//!
//! Full-path lookup deliberately precedes basename normalization so that two
//! different helpers sharing a basename cannot shadow each other.

use std::path::Path;

/// A documented legacy behavior profile.
///
/// Compatibility preserves arguments, output destinations, exit status, and
/// variant-specific formatting. It does not preserve injection, unsafe path
/// handling, or dependence on unavailable interpreters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompatibilityProfile {
    Canonical,
    PublicHandoffInit,
    PublicHandoffDetect,
    HelperHandoffDetect,
    PublicHandoffMigrate,
    HelperHandoffMigrate,
    PublicHandoffDatabase,
    HelperHandoffDatabase,
    CachedValerieReconcile,
    BundledValerieReconcile,
    ContextDocsDispatcher,
    ShellContextDocs,
    NushellContextDocs,
    NushellDiagrams,
    PythonDiagrams,
    JqSchemaInference,
    CodexSync,
    GitPreCommit,
    GitPrePush,
    GitPostCommit,
}

impl CompatibilityProfile {
    /// Every profile, in declaration order.
    pub const ALL: [Self; 20] = [
        Self::Canonical,
        Self::PublicHandoffInit,
        Self::PublicHandoffDetect,
        Self::HelperHandoffDetect,
        Self::PublicHandoffMigrate,
        Self::HelperHandoffMigrate,
        Self::PublicHandoffDatabase,
        Self::HelperHandoffDatabase,
        Self::CachedValerieReconcile,
        Self::BundledValerieReconcile,
        Self::ContextDocsDispatcher,
        Self::ShellContextDocs,
        Self::NushellContextDocs,
        Self::NushellDiagrams,
        Self::PythonDiagrams,
        Self::JqSchemaInference,
        Self::CodexSync,
        Self::GitPreCommit,
        Self::GitPrePush,
        Self::GitPostCommit,
    ];

    /// Kebab-case name, used on the command line and in reports.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Canonical => "canonical",
            Self::PublicHandoffInit => "public-handoff-init",
            Self::PublicHandoffDetect => "public-handoff-detect",
            Self::HelperHandoffDetect => "helper-handoff-detect",
            Self::PublicHandoffMigrate => "public-handoff-migrate",
            Self::HelperHandoffMigrate => "helper-handoff-migrate",
            Self::PublicHandoffDatabase => "public-handoff-database",
            Self::HelperHandoffDatabase => "helper-handoff-database",
            Self::CachedValerieReconcile => "cached-valerie-reconcile",
            Self::BundledValerieReconcile => "bundled-valerie-reconcile",
            Self::ContextDocsDispatcher => "context-docs-dispatcher",
            Self::ShellContextDocs => "shell-context-docs",
            Self::NushellContextDocs => "nushell-context-docs",
            Self::NushellDiagrams => "nushell-diagrams",
            Self::PythonDiagrams => "python-diagrams",
            Self::JqSchemaInference => "jq-schema-inference",
            Self::CodexSync => "codex-sync",
            Self::GitPreCommit => "git-pre-commit",
            Self::GitPrePush => "git-pre-push",
            Self::GitPostCommit => "git-post-commit",
        }
    }

    /// Parse a kebab-case profile name.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == value)
    }

    /// The exact-path alias table from the approved design.
    ///
    /// `bin/infer-json-schema.jq` is deliberately absent: it never had a
    /// working direct executable contract, since its supported invocation was
    /// `jq -f`, which supplies no argv-0 for a multicall binary to inspect.
    pub fn for_invocation_path(invocation: &Path) -> Option<Self> {
        let normalized = normalize_separators(&invocation.to_string_lossy());
        let trimmed = normalized.trim_start_matches('/');
        Some(match trimmed {
            "bin/handoff-init" => Self::PublicHandoffInit,
            "bin/handoff-detect" => Self::PublicHandoffDetect,
            "skills/handoff/helpers/handoff-detect.sh" => Self::HelperHandoffDetect,
            "bin/migrate-handoff" => Self::PublicHandoffMigrate,
            "skills/handoff/helpers/migrate-handoff.sh" => Self::HelperHandoffMigrate,
            "bin/handoff-db" => Self::PublicHandoffDatabase,
            "skills/handoff/helpers/handoff-db.sh" => Self::HelperHandoffDatabase,
            "bin/handoff-reconcile" => Self::CachedValerieReconcile,
            "bin/generate-ctx-docs" => Self::ContextDocsDispatcher,
            "skills/handoff/helpers/generate-ctx-docs.sh" => Self::ShellContextDocs,
            "skills/handoff/helpers/generate-ctx-docs.nu" => Self::NushellContextDocs,
            "skills/handover/helpers/generate-diagrams.nu" => Self::NushellDiagrams,
            "skills/handover/helpers/generate-diagrams.py" => Self::PythonDiagrams,
            "infer-json-schema" => Self::JqSchemaInference,
            "bin/sync-codex" => Self::CodexSync,
            ".githooks/pre-commit" => Self::GitPreCommit,
            ".githooks/pre-push" => Self::GitPrePush,
            ".githooks/post-commit" => Self::GitPostCommit,
            _ => return None,
        })
    }

    /// Basename fallback, used when no full path matched.
    ///
    /// Strips one case-insensitive `.exe` suffix, because Windows installs
    /// command aliases as `<alias>.exe`.
    pub fn for_executable_name(name: &str) -> Option<Self> {
        let base = name
            .rsplit('/')
            .next()
            .unwrap_or(name)
            .rsplit('\\')
            .next()
            .unwrap_or(name);
        let stem = if base.len() > 4 && base[base.len() - 4..].eq_ignore_ascii_case(".exe") {
            &base[..base.len() - 4]
        } else {
            base
        };
        Some(match stem {
            "handoff-init" => Self::PublicHandoffInit,
            "handoff-detect" => Self::PublicHandoffDetect,
            "migrate-handoff" => Self::PublicHandoffMigrate,
            "handoff-db" => Self::PublicHandoffDatabase,
            "handoff-reconcile" => Self::CachedValerieReconcile,
            "generate-ctx-docs" => Self::ContextDocsDispatcher,
            "infer-json-schema" => Self::JqSchemaInference,
            "sync-codex" => Self::CodexSync,
            "pre-commit" => Self::GitPreCommit,
            "pre-push" => Self::GitPrePush,
            "post-commit" => Self::GitPostCommit,
            _ => return None,
        })
    }
}

fn normalize_separators(value: &str) -> String {
    value.replace('\\', "/")
}

impl std::fmt::Display for CompatibilityProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilitySelection {
    pub profile: CompatibilityProfile,
    pub source: CompatibilitySource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilitySource {
    Explicit,
    InvocationPath,
    ExecutableName,
    Default,
}

/// Resolve the profile for an invocation, honouring the documented precedence.
pub fn resolve_compatibility(
    invocation: &Path,
    explicit: Option<CompatibilityProfile>,
) -> CompatibilitySelection {
    if let Some(profile) = explicit {
        return CompatibilitySelection {
            profile,
            source: CompatibilitySource::Explicit,
        };
    }
    if let Some(profile) = CompatibilityProfile::for_invocation_path(invocation) {
        return CompatibilitySelection {
            profile,
            source: CompatibilitySource::InvocationPath,
        };
    }
    let name = invocation.to_string_lossy();
    if let Some(profile) = CompatibilityProfile::for_executable_name(&name) {
        return CompatibilitySelection {
            profile,
            source: CompatibilitySource::ExecutableName,
        };
    }
    CompatibilitySelection {
        profile: CompatibilityProfile::Canonical,
        source: CompatibilitySource::Default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn all_twenty_profiles_are_distinct_and_kebab_case() {
        assert_eq!(CompatibilityProfile::ALL.len(), 20);
        let mut names: Vec<&str> = CompatibilityProfile::ALL
            .iter()
            .map(|p| p.as_str())
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 20, "profile names must be unique");
        for name in names {
            assert!(
                name.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{name} must be lowercase kebab-case"
            );
        }
    }

    #[test]
    fn every_profile_round_trips_through_parse() {
        for profile in CompatibilityProfile::ALL {
            assert_eq!(CompatibilityProfile::parse(profile.as_str()), Some(profile));
        }
    }

    #[test]
    fn explicit_wins_over_everything() {
        let selection = resolve_compatibility(
            &PathBuf::from("bin/handoff-init"),
            Some(CompatibilityProfile::JqSchemaInference),
        );
        assert_eq!(selection.profile, CompatibilityProfile::JqSchemaInference);
        assert_eq!(selection.source, CompatibilitySource::Explicit);
    }

    #[test]
    fn full_path_beats_basename() {
        // The helper and the public command share a basename; the path decides.
        let selection = resolve_compatibility(
            &PathBuf::from("skills/handoff/helpers/handoff-detect.sh"),
            None,
        );
        assert_eq!(selection.profile, CompatibilityProfile::HelperHandoffDetect);
        assert_eq!(selection.source, CompatibilitySource::InvocationPath);

        let selection = resolve_compatibility(&PathBuf::from("bin/handoff-detect"), None);
        assert_eq!(selection.profile, CompatibilityProfile::PublicHandoffDetect);
        assert_eq!(selection.source, CompatibilitySource::InvocationPath);
    }

    #[test]
    fn in_repo_handoff_detect_is_the_helper_profile() {
        // Named explicitly in the plan as a regression case.
        let selection = resolve_compatibility(&PathBuf::from("/repo/bin/handoff-detect"), None);
        assert_eq!(selection.profile, CompatibilityProfile::PublicHandoffDetect);
    }

    #[test]
    fn windows_separators_resolve() {
        let selection = resolve_compatibility(
            &PathBuf::from("skills\\handover\\helpers\\generate-diagrams.nu"),
            None,
        );
        assert_eq!(selection.profile, CompatibilityProfile::NushellDiagrams);
        assert_eq!(selection.source, CompatibilitySource::InvocationPath);
    }

    #[test]
    fn one_case_insensitive_exe_suffix_is_stripped() {
        for name in ["sync-codex.exe", "sync-codex.EXE", "sync-codex.Exe"] {
            let selection = resolve_compatibility(&PathBuf::from(name), None);
            assert_eq!(selection.profile, CompatibilityProfile::CodexSync, "{name}");
            assert_eq!(selection.source, CompatibilitySource::ExecutableName);
        }
    }

    #[test]
    fn unknown_invocation_falls_back_to_canonical() {
        let selection = resolve_compatibility(&PathBuf::from("atelier"), None);
        assert_eq!(selection.profile, CompatibilityProfile::Canonical);
        assert_eq!(selection.source, CompatibilitySource::Default);
    }

    #[test]
    fn infer_json_schema_jq_is_not_a_direct_alias() {
        // Documented: it has no argv-0 contract and stays a LegacyArtifact.
        assert_eq!(
            CompatibilityProfile::for_invocation_path(&PathBuf::from("bin/infer-json-schema.jq")),
            None
        );
        // But the supported invocation still maps.
        assert_eq!(
            CompatibilityProfile::for_invocation_path(&PathBuf::from("infer-json-schema")),
            Some(CompatibilityProfile::JqSchemaInference)
        );
    }

    #[test]
    fn only_one_exe_suffix_is_stripped() {
        // A double suffix is not a Windows alias and must not resolve.
        assert_eq!(
            CompatibilityProfile::for_executable_name("sync-codex.exe.exe"),
            None
        );
    }
}
