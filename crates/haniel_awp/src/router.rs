// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_awp::router — AWP sovereign URI parser and request dispatcher
//
// AWP URI format:  awp://<host>/<path>[?<query>]
// Sovereign Cache Doctrine: every dispatched request respects cache semantics.
// HERALD integration: router hands off to HERALD for threat gate before fetch.
// YouTube: detected URLs are intercepted and routed to YoutubeCompat.

use crate::registry::{AwpRegistry, ResourceKind};
use crate::response::{AwpResponse, AwpStatus};
use crate::youtube::{YoutubeCompat, YoutubeResolution};
use crate::AwpError;

/// A parsed AWP route
#[derive(Debug, Clone, PartialEq)]
pub struct AwpRoute {
    /// Scheme — always "awp" for sovereign URIs; "https" for legacy passthrough
    pub scheme: String,
    /// Host segment (e.g. "aieonyx", "youtube.com")
    pub host: String,
    /// Path after host (e.g. "home", "watch")
    pub path: String,
    /// Optional raw query string
    pub query: Option<String>,
    /// Whether this is an external (legacy web) URL
    pub is_external: bool,
}

impl AwpRoute {
    /// Parse a raw URI string into an AwpRoute.
    ///
    /// Accepts:
    ///   awp://aieonyx/home
    ///   awp://aieonyx/user/profile?id=42
    ///   https://www.youtube.com/watch?v=dQw4w9WgXcQ
    ///   https://youtu.be/dQw4w9WgXcQ
    ///   https://example.com/page
    pub fn parse(uri: &str) -> Result<Self, AwpError> {
        let uri = uri.trim();
        if uri.is_empty() {
            return Err(AwpError::MalformedUri("empty URI".to_string()));
        }

        // Split scheme
        let (scheme, rest) = if let Some(pos) = uri.find("://") {
            let s = &uri[..pos];
            let r = &uri[pos + 3..];
            (s.to_lowercase(), r)
        } else {
            return Err(AwpError::MalformedUri(
                format!("missing scheme in URI: {}", uri),
            ));
        };

        if scheme != "awp" && scheme != "https" && scheme != "http" {
            return Err(AwpError::MalformedUri(
                format!("unsupported scheme: {}", scheme),
            ));
        }

        // Split host / path
        let (host_raw, path_query) = if let Some(pos) = rest.find('/') {
            (&rest[..pos], &rest[pos + 1..])
        } else {
            // No slash → entire rest is host, path is empty
            (rest, "")
        };

        // Strip www. prefix from host
        let host = host_raw
            .trim_start_matches("www.")
            .to_lowercase();

        // Split path / query
        let (path, query) = if let Some(pos) = path_query.find('?') {
            let p = path_query[..pos].to_string();
            let q = path_query[pos + 1..].to_string();
            (p, if q.is_empty() { None } else { Some(q) })
        } else {
            (path_query.to_string(), None)
        };

        let is_external = scheme != "awp";

        Ok(Self { scheme, host, path, query, is_external })
    }

    /// Return the full canonical form of this route
    pub fn canonical(&self) -> String {
        let base = format!("{}://{}/{}", self.scheme, self.host, self.path);
        if let Some(q) = &self.query {
            format!("{}?{}", base, q)
        } else {
            base
        }
    }

    /// Extract a query parameter value by key
    pub fn query_param(&self, key: &str) -> Option<&str> {
        let q = self.query.as_deref()?;
        for part in q.split('&') {
            if let Some(eq) = part.find('=') {
                if &part[..eq] == key {
                    return Some(&part[eq + 1..]);
                }
            }
        }
        None
    }
}

/// AWP request dispatcher
///
/// Handles:
///   1. AWP sovereign URIs → AwpRegistry lookup → AwpResponse
///   2. YouTube URLs → YoutubeCompat → LUMEN media stream route
///   3. Other external URLs → passthrough notice (HERALD would gate these)
pub struct AwpRouter {
    registry: AwpRegistry,
    youtube:  YoutubeCompat,
}

impl AwpRouter {
    pub fn new() -> Self {
        Self {
            registry: AwpRegistry::new(),
            youtube:  YoutubeCompat::new(),
        }
    }

    /// Dispatch a raw URI string to an AwpResponse
    pub fn dispatch(&self, uri: &str) -> AwpResponse {
        let route = match AwpRoute::parse(uri) {
            Ok(r)  => r,
            Err(_) => return AwpResponse::not_found(uri),
        };

        if route.is_external {
            // YouTube sovereign intercept
            if self.youtube.is_youtube_url(&route.host) {
                return self.dispatch_youtube(&route);
            }
            // All other external URLs: HERALD would gate; return passthrough marker
            return self.external_passthrough(&route);
        }

        // AWP sovereign routing
        self.dispatch_awp(&route)
    }

    // ── AWP sovereign dispatch ────────────────────────────────────────────────

    fn dispatch_awp(&self, route: &AwpRoute) -> AwpResponse {
        // Redirect: path is empty → default to home
        let path = if route.path.is_empty() { "home" } else { route.path.as_str() };

        // Use lookup() (not resolve()) so a Redirect resource is seen as-is,
        // before resolve()'s own redirect-following kicks in.
        match self.registry.lookup(path) {
            Some(resource) => {
                match &resource.kind {
                    ResourceKind::Redirect(target) => {
                        AwpResponse::redirect(path, target)
                    }
                    ResourceKind::MediaStream => {
                        let body = format!(
                            "<axbw><title>{}</title><media-stream path=\"{}\"/></axbw>",
                            resource.title, path
                        ).into_bytes();
                        AwpResponse::media_stream(path, body)
                    }
                    _ => {
                        AwpResponse::ok(path, &resource.kind, resource.body.clone())
                    }
                }
            }
            None => AwpResponse::not_found(path),
        }
    }

    // ── YouTube sovereign intercept ───────────────────────────────────────────

    fn dispatch_youtube(&self, route: &AwpRoute) -> AwpResponse {
        let full_url = route.canonical();

        match self.youtube.extract_request(&full_url) {
            Ok(yt_req) => {
                // Build an AWP media stream response routed to LUMEN
                let stream_path = format!("media/youtube/{}", yt_req.video_id);
                let body = format!(
                    concat!(
                        "<axbw>",
                        "<title>{title}</title>",
                        "<media-stream type=\"youtube\"",
                        " video-id=\"{id}\"",
                        " resolution=\"{res}\"",
                        " tracking-stripped=\"true\"",
                        " sovereignty=\"LUMEN-routed\"/>",
                        "</axbw>"
                    ),
                    title = yt_req.video_id,
                    id    = yt_req.video_id,
                    res   = resolution_str(&yt_req.preferred_resolution),
                ).into_bytes();

                let mut resp = AwpResponse::media_stream(&stream_path, body);
                // Tag the response so LUMEN knows the video ID
                resp.headers.sovereignty = format!(
                    "AIEONYX/sovereign-boundary-enforced; youtube-id={}; tracking=stripped",
                    yt_req.video_id
                );
                resp
            }
            Err(e) => {
                let body = format!(
                    "<axbw><title>YouTube Error</title><p>{}</p></axbw>", e
                ).into_bytes();
                AwpResponse {
                    path:    route.canonical(),
                    headers: crate::response::AwpResponseHeaders::new(
                        "media/youtube/error",
                        &ResourceKind::MediaStream,
                    ),
                    body,
                    status:  AwpStatus::NotFound,
                }
            }
        }
    }

    // ── External passthrough notice ───────────────────────────────────────────

    fn external_passthrough(&self, route: &AwpRoute) -> AwpResponse {
        // HERALD gates this before actual fetch; we return a sovereign notice
        let path = format!("external/{}", route.host);
        let body = format!(
            concat!(
                "<axbw>",
                "<title>External Resource</title>",
                "<p>Host: {host}</p>",
                "<p>Path: {path}</p>",
                "<p>This request will pass through HERALD threat gate.</p>",
                "</axbw>"
            ),
            host = route.host,
            path = route.path,
        ).into_bytes();

        AwpResponse {
            path: path.clone(),
            headers: crate::response::AwpResponseHeaders::new(
                &path,
                &ResourceKind::AxbwPage,
            ),
            body,
            status: AwpStatus::Ok,
        }
    }
}

impl Default for AwpRouter {
    fn default() -> Self { Self::new() }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn resolution_str(r: &YoutubeResolution) -> &'static str {
    match r {
        YoutubeResolution::R144p  => "144p",
        YoutubeResolution::R360p  => "360p",
        YoutubeResolution::R480p  => "480p",
        YoutubeResolution::R720p  => "720p",
        YoutubeResolution::R1080p => "1080p",
        YoutubeResolution::R1440p => "1440p",
        YoutubeResolution::R2160p => "2160p",
        YoutubeResolution::Auto   => "auto",
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── AwpRoute::parse ──

    #[test]
    fn parse_awp_home() {
        let r = AwpRoute::parse("awp://aieonyx/home").unwrap();
        assert_eq!(r.scheme, "awp");
        assert_eq!(r.host,   "aieonyx");
        assert_eq!(r.path,   "home");
        assert!(r.query.is_none());
        assert!(!r.is_external);
    }

    #[test]
    fn parse_awp_with_query() {
        let r = AwpRoute::parse("awp://aieonyx/user/profile?id=42").unwrap();
        assert_eq!(r.path, "user/profile");
        assert_eq!(r.query_param("id"), Some("42"));
    }

    #[test]
    fn parse_awp_no_path() {
        let r = AwpRoute::parse("awp://aieonyx").unwrap();
        assert_eq!(r.host, "aieonyx");
        assert_eq!(r.path, "");
    }

    #[test]
    fn parse_https_external() {
        let r = AwpRoute::parse("https://example.com/page").unwrap();
        assert_eq!(r.scheme, "https");
        assert!(r.is_external);
        assert_eq!(r.host,   "example.com");
        assert_eq!(r.path,   "page");
    }

    #[test]
    fn parse_strips_www() {
        let r = AwpRoute::parse("https://www.youtube.com/watch?v=abc123").unwrap();
        assert_eq!(r.host, "youtube.com");
        assert_eq!(r.query_param("v"), Some("abc123"));
    }

    #[test]
    fn parse_empty_uri_error() {
        assert!(AwpRoute::parse("").is_err());
    }

    #[test]
    fn parse_missing_scheme_error() {
        assert!(AwpRoute::parse("aieonyx/home").is_err());
    }

    #[test]
    fn parse_unsupported_scheme_error() {
        assert!(AwpRoute::parse("ftp://files.example.com/data").is_err());
    }

    #[test]
    fn query_param_multi() {
        let r = AwpRoute::parse("awp://aieonyx/search?q=sovereign&page=2").unwrap();
        assert_eq!(r.query_param("q"),    Some("sovereign"));
        assert_eq!(r.query_param("page"), Some("2"));
        assert_eq!(r.query_param("missing"), None);
    }

    #[test]
    fn canonical_round_trip() {
        let uri = "awp://aieonyx/home";
        let r = AwpRoute::parse(uri).unwrap();
        assert_eq!(r.canonical(), uri);
    }

    #[test]
    fn canonical_with_query() {
        let uri = "awp://aieonyx/search?q=iam";
        let r = AwpRoute::parse(uri).unwrap();
        assert_eq!(r.canonical(), uri);
    }

    // ── AwpRouter::dispatch — AWP sovereign paths ──

    fn router() -> AwpRouter { AwpRouter::new() }

    #[test]
    fn dispatch_awp_home_ok() {
        let r = router().dispatch("awp://aieonyx/home");
        assert_eq!(r.status, AwpStatus::Ok);
    }

    #[test]
    fn dispatch_awp_aegis_ok() {
        let r = router().dispatch("awp://aieonyx/aegis");
        assert_eq!(r.status, AwpStatus::Ok);
    }

    #[test]
    fn dispatch_awp_iam_ok() {
        let r = router().dispatch("awp://aieonyx/iam");
        assert_eq!(r.status, AwpStatus::Ok);
    }

    #[test]
    fn dispatch_awp_vault_ok() {
        let r = router().dispatch("awp://aieonyx/vault");
        assert_eq!(r.status, AwpStatus::Ok);
    }

    #[test]
    fn dispatch_awp_sentinel_ok() {
        let r = router().dispatch("awp://aieonyx/sentinel");
        assert_eq!(r.status, AwpStatus::Ok);
    }

    #[test]
    fn dispatch_awp_db_ok() {
        let r = router().dispatch("awp://aieonyx/db");
        assert_eq!(r.status, AwpStatus::Ok);
    }

    #[test]
    fn dispatch_awp_not_found() {
        let r = router().dispatch("awp://aieonyx/does-not-exist");
        assert_eq!(r.status, AwpStatus::NotFound);
    }

    #[test]
    fn dispatch_awp_empty_path_routes_to_home() {
        let r = router().dispatch("awp://aieonyx");
        assert_eq!(r.status, AwpStatus::Ok);
    }

    #[test]
    fn dispatch_awp_redirect_start() {
        // "start" redirects to "home"
        let r = router().dispatch("awp://aieonyx/start");
        assert!(matches!(r.status, AwpStatus::Redirect(_)));
    }

    #[test]
    fn dispatch_awp_media_stream() {
        let r = router().dispatch("awp://aieonyx/media/stream");
        assert_eq!(r.status, AwpStatus::MediaStream);
    }

    #[test]
    fn dispatch_awp_consent_body_has_key_phrase() {
        let r = router().dispatch("awp://aieonyx/consent");
        assert!(r.body_as_str().contains("give the key away"));
    }

    // ── AwpRouter::dispatch — YouTube intercept ──

    #[test]
    fn dispatch_youtube_watch_returns_media_stream() {
        let r = router().dispatch("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert_eq!(r.status, AwpStatus::MediaStream);
    }

    #[test]
    fn dispatch_youtube_short_url_returns_media_stream() {
        let r = router().dispatch("https://youtu.be/dQw4w9WgXcQ");
        assert_eq!(r.status, AwpStatus::MediaStream);
    }

    #[test]
    fn dispatch_youtube_body_contains_video_id() {
        let r = router().dispatch("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert!(r.body_as_str().contains("dQw4w9WgXcQ"));
    }

    #[test]
    fn dispatch_youtube_sovereignty_header_stripped() {
        let r = router().dispatch("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert!(r.headers.sovereignty.contains("tracking=stripped"));
    }

    #[test]
    fn dispatch_youtube_embed_returns_media_stream() {
        let r = router().dispatch("https://www.youtube.com/embed/dQw4w9WgXcQ");
        assert_eq!(r.status, AwpStatus::MediaStream);
    }

    // ── AwpRouter::dispatch — external passthrough ──

    #[test]
    fn dispatch_external_non_youtube_returns_ok() {
        let r = router().dispatch("https://example.com/page");
        assert_eq!(r.status, AwpStatus::Ok);
    }

    #[test]
    fn dispatch_external_body_contains_herald_notice() {
        let r = router().dispatch("https://example.com/page");
        assert!(r.body_as_str().contains("HERALD"));
    }

    // ── AWP response header checks ──

    #[test]
    fn home_response_has_awp_version() {
        let r = router().dispatch("awp://aieonyx/home");
        assert!(!r.headers.awp_version.is_empty());
    }

    #[test]
    fn home_response_has_sovereignty_header() {
        let r = router().dispatch("awp://aieonyx/home");
        assert!(r.headers.sovereignty.contains("sovereign"));
    }

    #[test]
    fn session_path_cache_is_no_store() {
        let r = router().dispatch("awp://aieonyx/session/token");
        assert!(r.headers.cache_control.contains("no-store"));
    }
}
