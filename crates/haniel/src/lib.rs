// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL — Sovereign Browser and Rendering Engine
// Orchestrator crate — wires all 8 modules

#![forbid(unsafe_code)]

pub use haniel_herald as herald;
pub use haniel_prism as prism;
pub use haniel_canvas as canvas;
pub use haniel_echo as echo;
pub use haniel_onyx as onyx;
pub use haniel_vault as vault;
pub use haniel_sentinel as sentinel;
pub use haniel_lumen as lumen;

/// HANIEL engine version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// HANIEL engine identity
pub const ENGINE_NAME: &str = "AIEONYX HANIEL";

/// HANIEL copyright notice
pub const COPYRIGHT: &str = "Copyright (c) 2026 Edison Lepiten / AIEONYX";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_name_is_correct() {
        assert_eq!(ENGINE_NAME, "AIEONYX HANIEL");
    }

    #[test]
    fn copyright_contains_edison_lepiten() {
        assert!(COPYRIGHT.contains("Edison Lepiten"));
        assert!(COPYRIGHT.contains("AIEONYX"));
    }

    #[test]
    fn version_is_set() {
        assert!(!VERSION.is_empty());
    }
}
