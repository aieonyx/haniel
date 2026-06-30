// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_awp::registry — AWP sovereign resource registry
//
// Replaces the stub AwpRegistry in herald::fetch.
// Every awp:// path resolves here first.
// VAULT is the CDN replacement — no third-party CDN for sovereign data.

/// Kind of AWP resource
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceKind {
    /// .axbw sovereign page
    AxbwPage,
    /// Static sovereign asset
    Asset,
    /// Live sovereign data feed
    Feed,
    /// Redirect to another AWP path
    Redirect(String),
    /// Media stream — routes to LUMEN
    MediaStream,
}

/// A registered AWP resource
#[derive(Debug, Clone)]
pub struct AwpResource {
    pub path:    String,
    pub kind:    ResourceKind,
    pub title:   String,
    pub body:    Vec<u8>,
}

impl AwpResource {
    pub fn axbw(path: &str, title: &str, body: &str) -> Self {
        Self {
            path:  path.to_string(),
            kind:  ResourceKind::AxbwPage,
            title: title.to_string(),
            body:  format!("<axbw><title>{}</title>{}</axbw>", title, body).into_bytes(),
        }
    }

    pub fn redirect(path: &str, target: &str) -> Self {
        Self {
            path:  path.to_string(),
            kind:  ResourceKind::Redirect(target.to_string()),
            title: format!("Redirect → {}", target),
            body:  Vec::new(),
        }
    }

    pub fn media_stream(path: &str, title: &str) -> Self {
        Self {
            path:  path.to_string(),
            kind:  ResourceKind::MediaStream,
            title: title.to_string(),
            body:  format!("<axbw><title>{}</title><media-stream/></axbw>", title).into_bytes(),
        }
    }
}

/// AWP sovereign resource registry
pub struct AwpRegistry {
    resources: Vec<AwpResource>,
}

impl AwpRegistry {
    /// Build the registry with all sovereign AWP paths
    pub fn new() -> Self {
        let mut r = Self { resources: Vec::new() };
        r.register_all();
        r
    }

    fn register(&mut self, res: AwpResource) {
        self.resources.push(res);
    }

    fn register_all(&mut self) {
        // ── Core sovereign pages ────────────────────────────────────────────
        self.register(AwpResource::axbw(
            "home",
            "AIEONYX Sovereign Home",
            "<h1>Welcome to AIEONYX</h1><p>Sovereign digital civilisation.</p>",
        ));
        self.register(AwpResource::axbw(
            "aegis",
            "Aegis Sovereign Threat Dashboard",
            "<h1>AIEONYX Aegis</h1><p>Collective P2P threat intelligence.</p>",
        ));
        self.register(AwpResource::axbw(
            "legacy",
            "Digital Legacy Platform",
            "<h1>AIEONYX Digital Legacy</h1><p>Your sovereign data inheritance system.</p>",
        ));
        self.register(AwpResource::axbw(
            "iam",
            "IAM — Intelligent Assistant to Man",
            "<h1>IAM</h1><p>Sovereign intelligence. Your mind, extended.</p>",
        ));
        self.register(AwpResource::axbw(
            "vault",
            "VAULT — Sovereign Cache Layer",
            "<h1>VAULT</h1><p>Sovereign CDN replacement. No third-party caching.</p>",
        ));
        self.register(AwpResource::axbw(
            "sentinel",
            "SENTINEL — seL4 Policy Enforcer",
            "<h1>SENTINEL</h1><p>seL4 capability enforcement. Zero trust, verified.</p>",
        ));
        self.register(AwpResource::axbw(
            "db",
            "EdisonDB Sovereign Database",
            "<h1>EdisonDB</h1><p>Sovereign storage. Critical / Personal / Noise tiers.</p>",
        ));

        // ── Media / stream paths ────────────────────────────────────────────
        self.register(AwpResource::media_stream("media/stream", "AWP Media Stream"));

        // ── Redirects ───────────────────────────────────────────────────────
        self.register(AwpResource::redirect("start", "home"));

        // ── System paths ────────────────────────────────────────────────────
        self.register(AwpResource::axbw(
            "about",
            "About AIEONYX HANIEL",
            "<h1>AIEONYX HANIEL</h1><p>Sovereign rendering engine. S4+i.</p>",
        ));
        self.register(AwpResource::axbw(
            "consent",
            "Sovereign Consent Doctrine",
            concat!(
                "<h1>Consent</h1>",
                "<p>AIEONYX cannot help you if you give the key away.</p>",
                "<p>Warnings never block — you are sovereign.</p>",
            ),
        ));
    }

    /// Look up a resource by exact path
    pub fn lookup(&self, path: &str) -> Option<&AwpResource> {
        self.resources.iter().find(|r| r.path == path)
    }

    /// All registered paths
    pub fn paths(&self) -> Vec<&str> {
        self.resources.iter().map(|r| r.path.as_str()).collect()
    }

    /// Total number of registered resources
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Resolve a path, following one level of redirect
    pub fn resolve(&self, path: &str) -> Option<&AwpResource> {
        let res = self.lookup(path)?;
        if let ResourceKind::Redirect(target) = &res.kind {
            return self.lookup(target);
        }
        Some(res)
    }
}

impl Default for AwpRegistry {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> AwpRegistry { AwpRegistry::new() }

    #[test]
    fn registry_not_empty() {
        assert!(!reg().is_empty());
    }

    #[test]
    fn home_registered() {
        let registry = reg();
        let r = registry.lookup("home");
        assert!(r.is_some());
        assert_eq!(r.unwrap().kind, ResourceKind::AxbwPage);
    }

    #[test]
    fn aegis_registered() {
        assert!(reg().lookup("aegis").is_some());
    }

    #[test]
    fn legacy_registered() {
        assert!(reg().lookup("legacy").is_some());
    }

    #[test]
    fn iam_registered() {
        assert!(reg().lookup("iam").is_some());
    }

    #[test]
    fn vault_registered() {
        assert!(reg().lookup("vault").is_some());
    }

    #[test]
    fn sentinel_registered() {
        assert!(reg().lookup("sentinel").is_some());
    }

    #[test]
    fn db_registered() {
        assert!(reg().lookup("db").is_some());
    }

    #[test]
    fn media_stream_registered() {
        let registry = reg();
        let r = registry.lookup("media/stream").unwrap();
        assert_eq!(r.kind, ResourceKind::MediaStream);
    }

    #[test]
    fn redirect_start_follows_to_home() {
        let registry = reg();
        let resolved = registry.resolve("start").unwrap();
        assert_eq!(resolved.path, "home");
    }

    #[test]
    fn unknown_path_is_none() {
        assert!(reg().lookup("does-not-exist").is_none());
    }

    #[test]
    fn axbw_body_contains_title() {
        let registry = reg();
        let r = registry.lookup("home").unwrap();
        let body = std::str::from_utf8(&r.body).unwrap();
        assert!(body.contains("AIEONYX Sovereign Home"));
    }

    #[test]
    fn consent_body_contains_key_phrase() {
        let registry = reg();
        let r = registry.lookup("consent").unwrap();
        let body = std::str::from_utf8(&r.body).unwrap();
        assert!(body.contains("give the key away"));
    }

    #[test]
    fn all_paths_non_empty() {
        for path in reg().paths() {
            assert!(!path.is_empty());
        }
    }

    #[test]
    fn registry_has_at_least_ten_resources() {
        assert!(reg().len() >= 10);
    }
}
