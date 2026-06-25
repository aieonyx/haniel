// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL HERALD — Network, Protocol, STS Threat Gate, ARPi
// HE-3 implementation

#![forbid(unsafe_code)]

pub mod arpi;
pub mod fetch;
pub mod header;
pub mod protocol;
pub mod threat;

pub use fetch::HeraldFetch;
pub use threat::Sts;
pub use arpi::ArpiResolver;

/// Protocol detected from URI scheme
#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Awp,
    Https,
    Http,
}

/// Threat verdict from STS gate
#[derive(Debug, Clone, PartialEq)]
pub enum ThreatVerdict {
    Clean,
    Flagged(ThreatReason),
    Blocked(ThreatReason),
}

/// Reason for threat classification
#[derive(Debug, Clone, PartialEq)]
pub enum ThreatReason {
    TrackerDomain,
    Typosquat { similarity: f32, target: String },
    MixedContent,
    CryptoDrainer,
    MalformedOrigin,
}

/// ARPi trust tier for an origin
#[derive(Debug, Clone, PartialEq)]
pub enum ArpiTier {
    Sovereign,
    Verified,
    Guarded,
    Hostile,
}

/// Fetch response
#[derive(Debug)]
pub struct FetchResponse {
    pub body:           Vec<u8>,
    pub content_type:   ContentType,
    pub threat_verdict: ThreatVerdict,
    pub arpi_tier:      ArpiTier,
}

/// Content type classification
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Axbw,
    HtmlSubset,
    Asset(AssetKind),
}

/// Asset sub-type
#[derive(Debug, Clone, PartialEq)]
pub enum AssetKind {
    Image,
    Font,
    StyleModule,
    Script,
}

/// Fetch error
#[derive(Debug)]
pub enum FetchError {
    Blocked(ThreatReason),
    NetworkFailure(String),
    InvalidUri(String),
    Timeout,
}

/// HERALD sovereign fetch trait
pub trait Herald: Send + Sync {
    fn fetch(&self, uri: &str) -> Result<FetchResponse, FetchError>;
    fn threat_gate(&self, uri: &str) -> ThreatVerdict;
    fn resolve_arpi(&self, uri: &str) -> ArpiTier;
    fn axon_client_header(&self, tier: &ArpiTier) -> String;
}

impl Herald for HeraldFetch {
    fn fetch(&self, uri: &str) -> Result<FetchResponse, FetchError> {
        self.fetch(uri)
    }

    fn threat_gate(&self, uri: &str) -> ThreatVerdict {
        self.sts.classify(uri)
    }

    fn resolve_arpi(&self, uri: &str) -> ArpiTier {
        let threat = self.sts.classify(uri);
        self.resolver.resolve(uri, &threat)
    }

    fn axon_client_header(&self, tier: &ArpiTier) -> String {
        ArpiResolver::axon_client_header(tier)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn herald_fetch_awp_aegis() {
        let h = HeraldFetch::new();
        let r = Herald::fetch(&h, "awp://aegis").unwrap();
        assert_eq!(r.arpi_tier, ArpiTier::Sovereign);
    }

    #[test]
    fn herald_threat_gate_tracker() {
        let h = HeraldFetch::new();
        assert!(matches!(
            Herald::threat_gate(&h, "https://doubleclick.net/"),
            ThreatVerdict::Blocked(ThreatReason::TrackerDomain)
        ));
    }

    #[test]
    fn herald_resolve_arpi_awp() {
        let h = HeraldFetch::new();
        assert_eq!(
            Herald::resolve_arpi(&h, "awp://home"),
            ArpiTier::Sovereign
        );
    }

    #[test]
    fn herald_resolve_arpi_https() {
        let h = HeraldFetch::new();
        assert_eq!(
            Herald::resolve_arpi(&h, "https://example.com"),
            ArpiTier::Verified
        );
    }

    #[test]
    fn herald_axon_client_header_contains_haniel() {
        let h = HeraldFetch::new();
        let header = Herald::axon_client_header(&h, &ArpiTier::Sovereign);
        assert!(header.contains("HANIEL"));
        assert!(header.contains("AIEONYX"));
    }
}
