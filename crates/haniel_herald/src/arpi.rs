// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_herald::arpi — ARPi sovereign identity tier resolution

use crate::{ArpiTier, ThreatVerdict};
use crate::threat::Sts;

pub struct ArpiResolver;

impl ArpiResolver {
    pub fn new() -> Self { Self }

    pub fn resolve(&self, uri: &str, threat: &ThreatVerdict) -> ArpiTier {
        if matches!(threat, ThreatVerdict::Blocked(_)) {
            return ArpiTier::Hostile;
        }
        if uri.starts_with("awp://") {
            return ArpiTier::Sovereign;
        }
        if matches!(threat, ThreatVerdict::Flagged(_)) {
            return ArpiTier::Guarded;
        }
        let origin = Sts::extract_origin(uri);
        if uri.starts_with("https://") && !origin.is_empty() {
            return ArpiTier::Verified;
        }
        ArpiTier::Guarded
    }

    pub fn axon_client_header(tier: &ArpiTier) -> String {
        let tier_str = match tier {
            ArpiTier::Sovereign => "sovereign",
            ArpiTier::Verified  => "verified",
            ArpiTier::Guarded   => "guarded",
            ArpiTier::Hostile   => "hostile",
        };
        format!("AXON-Client/1.0 HANIEL/0.1 ARPi-Tier/{} AIEONYX/sovereign", tier_str)
    }
}

impl Default for ArpiResolver {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ThreatReason;

    fn r() -> ArpiResolver { ArpiResolver::new() }

    #[test]
    fn awp_is_sovereign() {
        assert_eq!(r().resolve("awp://aegis", &ThreatVerdict::Clean), ArpiTier::Sovereign);
    }

    #[test]
    fn https_clean_is_verified() {
        assert_eq!(r().resolve("https://example.com", &ThreatVerdict::Clean), ArpiTier::Verified);
    }

    #[test]
    fn http_clean_is_guarded() {
        assert_eq!(r().resolve("http://example.com", &ThreatVerdict::Clean), ArpiTier::Guarded);
    }

    #[test]
    fn blocked_is_hostile() {
        assert_eq!(
            r().resolve("https://t.com", &ThreatVerdict::Blocked(ThreatReason::TrackerDomain)),
            ArpiTier::Hostile
        );
    }

    #[test]
    fn flagged_is_guarded() {
        assert_eq!(
            r().resolve("https://gooogle.com", &ThreatVerdict::Flagged(
                ThreatReason::Typosquat { similarity: 0.9, target: "google.com".into() }
            )),
            ArpiTier::Guarded
        );
    }

    #[test]
    fn axon_client_header_sovereign() {
        let h = ArpiResolver::axon_client_header(&ArpiTier::Sovereign);
        assert!(h.contains("ARPi-Tier/sovereign"));
        assert!(h.contains("AIEONYX/sovereign"));
        assert!(h.contains("HANIEL"));
    }

    #[test]
    fn axon_client_header_verified() {
        let h = ArpiResolver::axon_client_header(&ArpiTier::Verified);
        assert!(h.contains("ARPi-Tier/verified"));
    }
}
