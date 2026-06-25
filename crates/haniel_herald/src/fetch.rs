// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_herald::fetch — Sovereign resource fetch pipeline

use crate::{
    FetchResponse, FetchError, Protocol,
    ThreatVerdict, ThreatReason,
};

use crate::threat::Sts;
use crate::arpi::ArpiResolver;
use crate::protocol::{detect, detect_content_type};

pub struct AwpRegistry;

impl AwpRegistry {
    pub fn serve(path: &str) -> Option<Vec<u8>> {
        match path {
            "aegis"  => Some(b"<axbw>Aegis Sovereign Threat Dashboard</axbw>".to_vec()),
            "legacy" => Some(b"<axbw>Digital Legacy Platform</axbw>".to_vec()),
            "home"   => Some(b"<axbw>AIEONYX Sovereign Home</axbw>".to_vec()),
            _        => None,
        }
    }
}

pub struct HeraldFetch {
    pub(crate) sts:      Sts,
    pub(crate) resolver: ArpiResolver,
}

impl HeraldFetch {
    pub fn new() -> Self {
        Self { sts: Sts::new(), resolver: ArpiResolver::new() }
    }

    pub fn fetch(&self, uri: &str) -> Result<FetchResponse, FetchError> {
        // Stage 1: Threat gate
        let threat = self.sts.classify(uri);
        if matches!(threat, ThreatVerdict::Blocked(_)) {
            return Err(FetchError::Blocked(
                match &threat {
                    ThreatVerdict::Blocked(r) => r.clone(),
                    _ => ThreatReason::MalformedOrigin,
                }
            ));
        }

        // Stage 2: ARPi tier
        let arpi_tier = self.resolver.resolve(uri, &threat);

        // Stage 3: Protocol
        let protocol = detect(uri);

        // Stage 4: Fetch
        let (body, ct_header) = match protocol {
            Protocol::Awp => {
                let path = uri.strip_prefix("awp://").unwrap_or("");
                let body = AwpRegistry::serve(path).unwrap_or_else(|| {
                    format!("<axbw>AWP resource not found: {}</axbw>", path).into_bytes()
                });
                (body, Some("application/axbw".to_string()))
            }
            Protocol::Https | Protocol::Http => {
                // axon_net HTTP fetch wired at HE-13
                let body = format!(
                    "<html><body>HANIEL fetched: {}</body></html>", uri
                ).into_bytes();
                (body, Some("text/html".to_string()))
            }
        };

        // Stage 5: Content type
        let content_type = detect_content_type(uri, ct_header.as_deref());

        Ok(FetchResponse { body, content_type, threat_verdict: threat, arpi_tier })
    }
}

impl Default for HeraldFetch {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArpiTier, ThreatVerdict, ThreatReason, ContentType};

    fn f() -> HeraldFetch { HeraldFetch::new() }

    #[test]
    fn fetch_awp_aegis_sovereign() {
        let r = f().fetch("awp://aegis").unwrap();
        assert_eq!(r.arpi_tier, ArpiTier::Sovereign);
        assert_eq!(r.content_type, ContentType::Axbw);
        assert!(!r.body.is_empty());
    }

    #[test]
    fn fetch_awp_unknown_not_found() {
        let r = f().fetch("awp://unknown-page").unwrap();
        assert!(String::from_utf8_lossy(&r.body).contains("not found"));
    }

    #[test]
    fn fetch_tracker_blocked() {
        assert!(matches!(
            f().fetch("https://google-analytics.com/collect"),
            Err(FetchError::Blocked(_))
        ));
    }

    #[test]
    fn fetch_crypto_drainer_blocked() {
        assert!(matches!(
            f().fetch("https://metamask-verify.io/connect"),
            Err(FetchError::Blocked(_))
        ));
    }

    #[test]
    fn fetch_https_clean_verified() {
        let r = f().fetch("https://example.com/page").unwrap();
        assert_eq!(r.arpi_tier, ArpiTier::Verified);
        assert_eq!(r.threat_verdict, ThreatVerdict::Clean);
    }

    #[test]
    fn fetch_awp_legacy_succeeds() {
        let r = f().fetch("awp://legacy").unwrap();
        assert_eq!(r.arpi_tier, ArpiTier::Sovereign);
        assert!(!r.body.is_empty());
    }

    #[test]
    fn awp_registry_known_paths() {
        assert!(AwpRegistry::serve("aegis").is_some());
        assert!(AwpRegistry::serve("legacy").is_some());
        assert!(AwpRegistry::serve("home").is_some());
    }

    #[test]
    fn awp_registry_unknown_none() {
        assert!(AwpRegistry::serve("nonexistent").is_none());
    }
}
