// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_awp::cache — Sovereign Cache Doctrine
//
// Doctrine: "The fastest query is the one you never run."
// Each AWP response carries cache semantics from day one.
// No cross-user cache bleed — sovereignty boundary strictly enforced.
// CDN layer replaced by HANIEL VAULT — no third-party CDN for sovereign data.

/// AWP cache directive for a response
#[derive(Debug, Clone, PartialEq)]
pub enum AwpCacheDirective {
    /// No caching — sensitive or personalised sovereign data
    NoStore,
    /// Private: user-scoped only — never shared across sessions
    Private { max_age_secs: u32 },
    /// Public: cacheable by VAULT layer — no user identity bound
    Public { max_age_secs: u32 },
    /// Immutable: static sovereign asset — cache until explicit invalidation
    Immutable { max_age_secs: u32 },
    /// Revalidate on every request with sovereign ETag
    MustRevalidate { max_age_secs: u32 },
}

/// Cache policy resolver — maps resource kind to directive
#[derive(Debug, Clone)]
pub struct AwpCachePolicy;

impl AwpCachePolicy {
    pub fn new() -> Self { Self }

    /// Resolve the appropriate cache directive for a resource path
    pub fn directive_for(&self, path: &str) -> AwpCacheDirective {
        let lower = path.to_lowercase();

        // Sovereign identity / session data — never cache
        if lower.contains("identity") || lower.contains("session")
            || lower.contains("key") || lower.contains("auth")
            || lower.contains("consent") {
            return AwpCacheDirective::NoStore;
        }

        // User-personalised data — private, short TTL
        if lower.starts_with("user/") || lower.starts_with("profile/")
            || lower.contains("personal") {
            return AwpCacheDirective::Private { max_age_secs: 300 }; // 5 min
        }

        // Static assets (fonts, images, manifests) — immutable, long TTL
        if lower.ends_with(".woff2") || lower.ends_with(".ttf")
            || lower.ends_with(".png")  || lower.ends_with(".svg")
            || lower.ends_with(".webp") {
            return AwpCacheDirective::Immutable { max_age_secs: 86_400 * 30 }; // 30 days
        }

        // AXBW sovereign pages — public, short TTL, revalidate
        if lower.ends_with(".axbw") {
            return AwpCacheDirective::MustRevalidate { max_age_secs: 60 };
        }

        // Default sovereign pages — public, moderate TTL
        AwpCacheDirective::Public { max_age_secs: 300 }
    }

    /// Format directive as AWP-Cache header value
    pub fn header_value(directive: &AwpCacheDirective) -> String {
        match directive {
            AwpCacheDirective::NoStore                      => "no-store, sovereign".to_string(),
            AwpCacheDirective::Private { max_age_secs }     =>
                format!("private, max-age={}, sovereign", max_age_secs),
            AwpCacheDirective::Public  { max_age_secs }     =>
                format!("public, max-age={}, sovereign", max_age_secs),
            AwpCacheDirective::Immutable { max_age_secs }   =>
                format!("public, max-age={}, immutable, sovereign", max_age_secs),
            AwpCacheDirective::MustRevalidate { max_age_secs } =>
                format!("must-revalidate, max-age={}, sovereign", max_age_secs),
        }
    }
}

impl Default for AwpCachePolicy {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> AwpCachePolicy { AwpCachePolicy::new() }

    #[test]
    fn identity_path_no_store() {
        assert_eq!(p().directive_for("user/identity/key"), AwpCacheDirective::NoStore);
    }

    #[test]
    fn session_path_no_store() {
        assert_eq!(p().directive_for("session/token"), AwpCacheDirective::NoStore);
    }

    #[test]
    fn auth_path_no_store() {
        assert_eq!(p().directive_for("auth/login"), AwpCacheDirective::NoStore);
    }

    #[test]
    fn consent_path_no_store() {
        assert_eq!(p().directive_for("consent/gdpr"), AwpCacheDirective::NoStore);
    }

    #[test]
    fn user_profile_private() {
        match p().directive_for("user/preferences") {
            AwpCacheDirective::Private { max_age_secs } => assert!(max_age_secs > 0),
            other => panic!("Expected Private, got {:?}", other),
        }
    }

    #[test]
    fn font_asset_immutable() {
        match p().directive_for("assets/sovereign.woff2") {
            AwpCacheDirective::Immutable { max_age_secs } => assert!(max_age_secs >= 86_400),
            other => panic!("Expected Immutable, got {:?}", other),
        }
    }

    #[test]
    fn image_asset_immutable() {
        match p().directive_for("assets/logo.png") {
            AwpCacheDirective::Immutable { max_age_secs } => assert!(max_age_secs >= 86_400),
            other => panic!("Expected Immutable, got {:?}", other),
        }
    }

    #[test]
    fn axbw_page_must_revalidate() {
        match p().directive_for("home.axbw") {
            AwpCacheDirective::MustRevalidate { max_age_secs } => assert!(max_age_secs > 0),
            other => panic!("Expected MustRevalidate, got {:?}", other),
        }
    }

    #[test]
    fn default_page_public() {
        match p().directive_for("aegis") {
            AwpCacheDirective::Public { max_age_secs } => assert!(max_age_secs > 0),
            other => panic!("Expected Public, got {:?}", other),
        }
    }

    #[test]
    fn no_store_header_value() {
        let h = AwpCachePolicy::header_value(&AwpCacheDirective::NoStore);
        assert!(h.contains("no-store"));
        assert!(h.contains("sovereign"));
    }

    #[test]
    fn public_header_value_contains_max_age() {
        let h = AwpCachePolicy::header_value(&AwpCacheDirective::Public { max_age_secs: 300 });
        assert!(h.contains("max-age=300"));
        assert!(h.contains("sovereign"));
    }

    #[test]
    fn immutable_header_value() {
        let d = AwpCacheDirective::Immutable { max_age_secs: 2_592_000 };
        let h = AwpCachePolicy::header_value(&d);
        assert!(h.contains("immutable"));
        assert!(h.contains("sovereign"));
    }
}
