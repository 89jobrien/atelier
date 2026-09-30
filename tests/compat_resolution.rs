//! Compatibility resolution contracts.
//!
//! These lock the documented precedence and the exact alias inventory, both of
//! which are relied on by installed aliases on three platforms.

use std::path::PathBuf;

use atelier::compat::{CompatibilityProfile, CompatibilitySource, resolve_compatibility};

fn select(path: &str) -> atelier::CompatibilitySelection {
    resolve_compatibility(&PathBuf::from(path), None)
}

#[test]
fn explicit_profile_takes_precedence() {
    let selection = resolve_compatibility(
        &PathBuf::from("bin/handoff-init"),
        Some(CompatibilityProfile::Canonical),
    );
    assert_eq!(selection.profile, CompatibilityProfile::Canonical);
    assert_eq!(selection.source, CompatibilitySource::Explicit);
}

#[test]
fn alias_inventory_matches_the_approved_table() {
    let expected = [
        ("bin/handoff-init", CompatibilityProfile::PublicHandoffInit),
        (
            "bin/handoff-detect",
            CompatibilityProfile::PublicHandoffDetect,
        ),
        (
            "skills/handoff/helpers/handoff-detect.sh",
            CompatibilityProfile::HelperHandoffDetect,
        ),
        (
            "bin/migrate-handoff",
            CompatibilityProfile::PublicHandoffMigrate,
        ),
        (
            "skills/handoff/helpers/migrate-handoff.sh",
            CompatibilityProfile::HelperHandoffMigrate,
        ),
        (
            "bin/handoff-db",
            CompatibilityProfile::PublicHandoffDatabase,
        ),
        (
            "skills/handoff/helpers/handoff-db.sh",
            CompatibilityProfile::HelperHandoffDatabase,
        ),
        (
            "bin/handoff-reconcile",
            CompatibilityProfile::CachedValerieReconcile,
        ),
        (
            "bin/generate-ctx-docs",
            CompatibilityProfile::ContextDocsDispatcher,
        ),
        (
            "skills/handoff/helpers/generate-ctx-docs.sh",
            CompatibilityProfile::ShellContextDocs,
        ),
        (
            "skills/handoff/helpers/generate-ctx-docs.nu",
            CompatibilityProfile::NushellContextDocs,
        ),
        (
            "skills/handover/helpers/generate-diagrams.nu",
            CompatibilityProfile::NushellDiagrams,
        ),
        (
            "skills/handover/helpers/generate-diagrams.py",
            CompatibilityProfile::PythonDiagrams,
        ),
        ("infer-json-schema", CompatibilityProfile::JqSchemaInference),
        ("bin/sync-codex", CompatibilityProfile::CodexSync),
        (".githooks/pre-commit", CompatibilityProfile::GitPreCommit),
        (".githooks/pre-push", CompatibilityProfile::GitPrePush),
        (".githooks/post-commit", CompatibilityProfile::GitPostCommit),
    ];

    for (invocation, profile) in expected {
        let selection = select(invocation);
        assert_eq!(selection.profile, profile, "invocation {invocation}");
        assert_eq!(
            selection.source,
            CompatibilitySource::InvocationPath,
            "invocation {invocation}"
        );
    }
}

#[test]
fn jq_script_is_not_a_direct_alias() {
    // `bin/infer-json-schema.jq` never supplied an argv-0 contract.
    assert_eq!(
        CompatibilityProfile::for_invocation_path(&PathBuf::from("bin/infer-json-schema.jq")),
        None
    );
}

#[test]
fn in_repo_handoff_detect_resolves_to_the_public_profile() {
    let selection = select("/repo/bin/handoff-detect");
    // The alias table is keyed by source-relative paths, so an absolute
    // invocation cannot match it directly. It still selects the right profile
    // through the basename fallback, which is what an installed alias under
    // any prefix needs.
    assert_eq!(selection.profile, CompatibilityProfile::PublicHandoffDetect);
    assert_eq!(selection.source, CompatibilitySource::ExecutableName);
}

#[test]
fn source_relative_path_matches_the_alias_table_directly() {
    let selection = select("bin/handoff-detect");
    assert_eq!(selection.profile, CompatibilityProfile::PublicHandoffDetect);
    assert_eq!(selection.source, CompatibilitySource::InvocationPath);
}

#[test]
fn windows_separators_resolve_the_same_profiles() {
    assert_eq!(
        select(".githooks\\pre-push").profile,
        CompatibilityProfile::GitPrePush
    );
    assert_eq!(
        select("bin\\handoff-init").profile,
        CompatibilityProfile::PublicHandoffInit
    );
}

#[test]
fn single_case_insensitive_exe_suffix_resolves() {
    for name in ["handoff-db.exe", "handoff-db.EXE"] {
        let selection = resolve_compatibility(&PathBuf::from(name), None);
        assert_eq!(
            selection.profile,
            CompatibilityProfile::PublicHandoffDatabase,
            "{name}"
        );
        assert_eq!(selection.source, CompatibilitySource::ExecutableName);
    }
}

#[test]
fn unknown_invocation_is_canonical() {
    let selection = select("/usr/local/bin/atelier");
    assert_eq!(selection.profile, CompatibilityProfile::Canonical);
    assert_eq!(selection.source, CompatibilitySource::Default);
}

#[test]
fn every_profile_serializes_as_lowercase_kebab_case() {
    for profile in CompatibilityProfile::ALL {
        let name = profile.as_str();
        assert!(!name.is_empty());
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "{name} is not kebab-case"
        );
        assert!(!name.starts_with('-') && !name.ends_with('-'));
    }
}

#[test]
fn profile_names_are_unique() {
    let mut names: Vec<&str> = CompatibilityProfile::ALL
        .iter()
        .map(|p| p.as_str())
        .collect();
    assert_eq!(names.len(), 20);
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), 20);
}
