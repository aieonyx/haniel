// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_herald::threat — Sovereign Threat Sensor gate

use crate::{ThreatVerdict, ThreatReason};

static TRACKER_DOMAINS: &[&str] = &[
    "google-analytics.com", "googletagmanager.com", "doubleclick.net",
    "googlesyndication.com", "facebook.com", "facebook.net",
    "connect.facebook.net", "analytics.twitter.com", "ads.twitter.com",
    "platform.twitter.com", "amazon-adsystem.com", "advertising.com",
    "adnxs.com", "scorecardresearch.com", "quantserve.com",
    "chartbeat.com", "hotjar.com", "criteo.com", "criteo.net",
    "outbrain.com", "taboola.com", "rubiconproject.com", "openx.net",
    "pubmatic.com", "casalemedia.com", "mathtag.com", "moatads.com",
    "adsafeprotected.com", "newrelic.com",
];

static CRYPTO_DRAINER_DOMAINS: &[&str] = &[
    "metamask-verify.io", "wallet-connect.app", "nft-claim.net",
];

pub struct Sts;

impl Sts {
    pub fn new() -> Self { Self }

    pub fn classify(&self, uri: &str) -> ThreatVerdict {
        let origin = Self::extract_origin(uri);
        if Self::is_crypto_drainer(&origin) {
            return ThreatVerdict::Blocked(ThreatReason::CryptoDrainer);
        }
        if Self::is_tracker(&origin) {
            return ThreatVerdict::Blocked(ThreatReason::TrackerDomain);
        }
        if let Some(reason) = Self::check_typosquat(&origin) {
            return ThreatVerdict::Flagged(reason);
        }
        if origin.is_empty() {
            return ThreatVerdict::Blocked(ThreatReason::MalformedOrigin);
        }
        ThreatVerdict::Clean
    }

    pub fn check_mixed_content(page_uri: &str, resource_uri: &str) -> Option<ThreatReason> {
        let page_https    = page_uri.starts_with("https://");
        let resource_http = resource_uri.starts_with("http://")
            && !resource_uri.starts_with("https://");
        if page_https && resource_http {
            Some(ThreatReason::MixedContent)
        } else {
            None
        }
    }

    pub fn extract_origin(uri: &str) -> String {
        let without_scheme = if let Some(s) = uri.strip_prefix("https://") { s }
            else if let Some(s) = uri.strip_prefix("http://") { s }
            else if let Some(s) = uri.strip_prefix("awp://") { s }
            else { uri };
        let host = without_scheme.split('/').next().unwrap_or("");
        let host = host.split(':').next().unwrap_or("");
        host.to_lowercase()
    }

    fn is_tracker(origin: &str) -> bool {
        for t in TRACKER_DOMAINS {
            if origin == *t || origin.ends_with(&format!(".{}", t)) {
                return true;
            }
        }
        false
    }

    fn is_crypto_drainer(origin: &str) -> bool {
        for d in CRYPTO_DRAINER_DOMAINS {
            if origin == *d || origin.ends_with(&format!(".{}", d)) {
                return true;
            }
        }
        false
    }

    fn check_typosquat(origin: &str) -> Option<ThreatReason> {
        static TARGETS: &[&str] = &[
            "google.com", "facebook.com", "amazon.com", "apple.com",
            "microsoft.com", "paypal.com", "bankofamerica.com",
            "chase.com", "wellsfargo.com", "coinbase.com",
            "binance.com", "metamask.io",
        ];
        for target in TARGETS {
            if origin == *target { return None; }
            let dist = levenshtein(origin, target);
            if dist > 0 && dist <= 2 {
                return Some(ThreatReason::Typosquat {
                    similarity: 1.0 - (dist as f32 / target.len() as f32),
                    target: target.to_string(),
                });
            }
        }
        None
    }
}

impl Default for Sts {
    fn default() -> Self { Self::new() }
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    if m == 0 { return n; }
    if n == 0 { return m; }
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }
    for (i, ca) in a.iter().enumerate().map(|(i,c)| (i+1, c)) {
        for (j, cb) in b.iter().enumerate().map(|(j,c)| (j+1, c)) {
            let cost = if ca == cb { 0 } else { 1 };
            dp[i][j] = (dp[i-1][j] + 1).min(dp[i][j-1] + 1).min(dp[i-1][j-1] + cost);
        }
    }
    dp[m][n]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sts() -> Sts { Sts::new() }

    #[test]
    fn clean_url_passes() {
        assert_eq!(sts().classify("https://example.com/page"), ThreatVerdict::Clean);
    }

    #[test]
    fn tracker_google_analytics_blocked() {
        assert_eq!(
            sts().classify("https://google-analytics.com/collect"),
            ThreatVerdict::Blocked(ThreatReason::TrackerDomain)
        );
    }

    #[test]
    fn tracker_doubleclick_blocked() {
        assert_eq!(
            sts().classify("https://doubleclick.net/pixel"),
            ThreatVerdict::Blocked(ThreatReason::TrackerDomain)
        );
    }

    #[test]
    fn tracker_subdomain_blocked() {
        assert_eq!(
            sts().classify("https://stats.hotjar.com/track"),
            ThreatVerdict::Blocked(ThreatReason::TrackerDomain)
        );
    }

    #[test]
    fn crypto_drainer_blocked() {
        assert_eq!(
            sts().classify("https://metamask-verify.io/connect"),
            ThreatVerdict::Blocked(ThreatReason::CryptoDrainer)
        );
    }

    #[test]
    fn typosquat_paypa1_flagged() {
        let result = sts().classify("https://paypa1.com/login");
        match result {
            ThreatVerdict::Flagged(ThreatReason::Typosquat { target, .. }) => {
                assert_eq!(target, "paypal.com");
            }
            _ => panic!("expected typosquat, got: {:?}", result),
        }
    }

    #[test]
    fn typosquat_gooogle_flagged() {
        let result = sts().classify("https://gooogle.com/search");
        match result {
            ThreatVerdict::Flagged(ThreatReason::Typosquat { target, .. }) => {
                assert_eq!(target, "google.com");
            }
            _ => panic!("expected typosquat, got: {:?}", result),
        }
    }

    #[test]
    fn exact_target_not_flagged() {
        assert_eq!(sts().classify("https://google.com/search"), ThreatVerdict::Clean);
    }

    #[test]
    fn awp_origin_clean() {
        assert_eq!(sts().classify("awp://aegis"), ThreatVerdict::Clean);
    }

    #[test]
    fn mixed_content_detected() {
        let r = Sts::check_mixed_content("https://secure.com/page", "http://cdn.com/s.js");
        assert!(matches!(r, Some(ThreatReason::MixedContent)));
    }

    #[test]
    fn no_mixed_content_both_https() {
        let r = Sts::check_mixed_content("https://secure.com/page", "https://cdn.com/s.js");
        assert!(r.is_none());
    }

    #[test]
    fn extract_origin_strips_scheme() {
        assert_eq!(Sts::extract_origin("https://example.com/path"), "example.com");
    }

    #[test]
    fn extract_origin_strips_port() {
        assert_eq!(Sts::extract_origin("https://example.com:8080/path"), "example.com");
    }

    #[test]
    fn levenshtein_identical() { assert_eq!(levenshtein("abc", "abc"), 0); }

    #[test]
    fn levenshtein_one_edit() { assert_eq!(levenshtein("abc", "abx"), 1); }

    #[test]
    fn levenshtein_empty() {
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("abc", ""), 3);
    }

    #[test]
    fn all_29_trackers_blocked() {
        let s = sts();
        for tracker in TRACKER_DOMAINS {
            let uri     = format!("https://{}/track", tracker);
            let verdict = s.classify(&uri);
            assert!(
                matches!(verdict, ThreatVerdict::Blocked(ThreatReason::TrackerDomain)),
                "not blocked: {}", tracker
            );
        }
    }
}
