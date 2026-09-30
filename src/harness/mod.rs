//! Harness identity and per-harness generation machinery.
//!
//! The three adapters Atelier ships together. `HarnessId` is deliberately
//! `Copy + Hash` because it keys ownership receipts, journal entries, and
//! parity reports.

pub mod generation;
pub mod ownership;

use std::fmt;

/// One of the supported agent harnesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HarnessId {
    Claude,
    Codex,
    OpenCode,
}

impl HarnessId {
    /// Stable lowercase name used in receipts, journal filenames, and the
    /// `receipts/<harness>-<scope>.json` layout.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::OpenCode => "opencode",
        }
    }

    /// Every harness, in installation order.
    pub const ALL: [Self; 3] = [Self::Claude, Self::Codex, Self::OpenCode];
}

impl fmt::Display for HarnessId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::HarnessId;

    #[test]
    fn names_are_stable_and_lowercase() {
        assert_eq!(HarnessId::Claude.as_str(), "claude");
        assert_eq!(HarnessId::Codex.as_str(), "codex");
        assert_eq!(HarnessId::OpenCode.as_str(), "opencode");
    }

    #[test]
    fn all_lists_three_harnesses_without_duplicates() {
        assert_eq!(HarnessId::ALL.len(), 3);
        let mut seen: Vec<&str> = HarnessId::ALL.iter().map(|h| h.as_str()).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 3);
    }

    #[test]
    fn display_matches_as_str() {
        for harness in HarnessId::ALL {
            assert_eq!(harness.to_string(), harness.as_str());
        }
    }
}
