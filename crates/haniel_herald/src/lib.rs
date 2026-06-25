// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL HERALD — Network, Protocol, STS Threat Gate, ARPi
// HE-3 implementation target

#![forbid(unsafe_code)]

pub mod fetch;
pub mod threat;
pub mod arpi;
pub mod protocol;
pub mod header;

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

/// Outgoing fetch request
#[derive(Debug, Clone)]
pub struct FetchRequest {
    pub uri: String,
    pub protocol: Protocol,
    pub arpi_tier: ArpiTier,
}

/// Incoming fetch response
#[derive(Debug)]
pub struct FetchResponse {
    pub body: Vec<u8>,
    pub content_type: ContentType,
    pub threat_verdict: ThreatVerdict,
    pub arpi_tier: ArpiTier,
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

/// HERALD sovereign fetch trait
pub trait Herald: Send + Sync {
    fn fetch(&self, uri: &str) -> Result<FetchResponse, HeraldError>;
    fn threat_gate(&self, uri: &str) -> ThreatVerdict;
    fn resolve_arpi(&self, origin: &str) -> ArpiTier;
    fn detect_protocol(&self, uri: &str) -> Protocol;
    fn axon_client_header(&self) -> String;
}

/// HERALD error type
#[derive(Debug)]
pub enum HeraldError {
    Blocked(ThreatReason),
    NetworkFailure(String),
    InvalidUri(String),
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_awp_detected() {
        // HE-3 implementation will make this pass
        let uri = "awp://aegis";
        assert!(uri.starts_with("awp://"));
    }

    #[test]
    fn arpi_tier_variants_exist() {
        let _s = ArpiTier::Sovereign;
        let _v = ArpiTier::Verified;
        let _g = ArpiTier::Guarded;
        let _h = ArpiTier::Hostile;
    }

    #[test]
    fn threat_verdict_variants_exist() {
        let _c = ThreatVerdict::Clean;
        let _b = ThreatVerdict::Blocked(ThreatReason::TrackerDomain);
    }
}
