// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_herald::header — AXON-Client sovereign header injection

use crate::ArpiTier;
use crate::arpi::ArpiResolver;

pub const AXON_CLIENT_HEADER_NAME: &str = "AXON-Client";

pub fn axon_client_header(tier: &ArpiTier) -> String {
    ArpiResolver::axon_client_header(tier)
}

pub fn sovereign_headers(tier: &ArpiTier) -> Vec<(String, String)> {
    vec![
        (AXON_CLIENT_HEADER_NAME.to_string(), axon_client_header(tier)),
        ("X-AIEONYX-Engine".to_string(),      "HANIEL/0.1".to_string()),
        ("X-AIEONYX-Sovereignty".to_string(), "sovereign-first".to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_name_correct() { assert_eq!(AXON_CLIENT_HEADER_NAME, "AXON-Client"); }

    #[test]
    fn sovereign_headers_count() {
        assert_eq!(sovereign_headers(&ArpiTier::Verified).len(), 3);
    }

    #[test]
    fn sovereign_headers_contain_axon_client() {
        let h = sovereign_headers(&ArpiTier::Sovereign);
        let v = h.iter().find(|(k,_)| k == "AXON-Client").unwrap();
        assert!(v.1.contains("AIEONYX"));
    }

    #[test]
    fn sovereign_headers_contain_engine() {
        let h = sovereign_headers(&ArpiTier::Verified);
        let v = h.iter().find(|(k,_)| k == "X-AIEONYX-Engine").unwrap();
        assert!(v.1.contains("HANIEL"));
    }
}
