// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_awp::response — AWP sovereign response with cache headers
//
// Every response carries AWP cache semantics from day one.
// No silent cache bleed across sovereignty boundary.

use crate::{AWP_CLIENT_HEADER, AWP_VERSION};
use crate::cache::AwpCachePolicy;
use crate::registry::ResourceKind;

/// AWP response headers (sovereign cache + identity)
#[derive(Debug, Clone)]
pub struct AwpResponseHeaders {
    pub awp_version:    String,
    pub cache_control:  String,
    pub axon_client:    String,
    pub content_type:   String,
    pub sovereignty:    String,
}

impl AwpResponseHeaders {
    pub fn new(path: &str, kind: &ResourceKind) -> Self {
        let policy = AwpCachePolicy::new();
        let directive = policy.directive_for(path);
        let cache_control = AwpCachePolicy::header_value(&directive);

        let content_type = match kind {
            ResourceKind::AxbwPage   => "application/axbw; charset=utf-8".to_string(),
            ResourceKind::Asset      => "application/octet-stream".to_string(),
            ResourceKind::Feed       => "application/axbw+feed; charset=utf-8".to_string(),
            ResourceKind::MediaStream => "application/x-awp-stream".to_string(),
            ResourceKind::Redirect(_) => "text/plain".to_string(),
        };

        Self {
            awp_version:  AWP_VERSION.to_string(),
            cache_control,
            axon_client:  AWP_CLIENT_HEADER.to_string(),
            content_type,
            sovereignty:  "AIEONYX/sovereign-boundary-enforced".to_string(),
        }
    }

    /// Emit all headers as key-value pairs
    pub fn as_pairs(&self) -> Vec<(&str, &str)> {
        vec![
            ("AWP-Version",         self.awp_version.as_str()),
            ("Cache-Control",       self.cache_control.as_str()),
            ("AXON-Client",         self.axon_client.as_str()),
            ("Content-Type",        self.content_type.as_str()),
            ("AIEONYX-Sovereignty", self.sovereignty.as_str()),
        ]
    }
}

/// Complete AWP response
#[derive(Debug)]
pub struct AwpResponse {
    pub path:    String,
    pub headers: AwpResponseHeaders,
    pub body:    Vec<u8>,
    pub status:  AwpStatus,
}

/// AWP response status
#[derive(Debug, Clone, PartialEq)]
pub enum AwpStatus {
    Ok,
    NotFound,
    Redirect(String),
    PolicyRejected,
    MediaStream,
}

impl AwpResponse {
    pub fn ok(path: &str, kind: &ResourceKind, body: Vec<u8>) -> Self {
        Self {
            path:    path.to_string(),
            headers: AwpResponseHeaders::new(path, kind),
            body,
            status:  AwpStatus::Ok,
        }
    }

    pub fn not_found(path: &str) -> Self {
        let body = format!(
            "<axbw><title>Not Found</title><p>AWP resource not found: {}</p></axbw>",
            path
        ).into_bytes();
        Self {
            path:    path.to_string(),
            headers: AwpResponseHeaders::new(path, &ResourceKind::AxbwPage),
            body,
            status:  AwpStatus::NotFound,
        }
    }

    pub fn redirect(path: &str, target: &str) -> Self {
        Self {
            path:    path.to_string(),
            headers: AwpResponseHeaders::new(path, &ResourceKind::Redirect(target.to_string())),
            body:    Vec::new(),
            status:  AwpStatus::Redirect(target.to_string()),
        }
    }

    pub fn media_stream(path: &str, body: Vec<u8>) -> Self {
        Self {
            path:    path.to_string(),
            headers: AwpResponseHeaders::new(path, &ResourceKind::MediaStream),
            body,
            status:  AwpStatus::MediaStream,
        }
    }

    pub fn is_ok(&self) -> bool { self.status == AwpStatus::Ok }

    pub fn body_as_str(&self) -> &str {
        std::str::from_utf8(&self.body).unwrap_or("[binary]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_response_is_ok() {
        let r = AwpResponse::ok("home", &ResourceKind::AxbwPage, b"<axbw/>".to_vec());
        assert!(r.is_ok());
        assert_eq!(r.status, AwpStatus::Ok);
    }

    #[test]
    fn not_found_status() {
        let r = AwpResponse::not_found("missing/page");
        assert_eq!(r.status, AwpStatus::NotFound);
        assert!(r.body_as_str().contains("not found"));
    }

    #[test]
    fn redirect_carries_target() {
        let r = AwpResponse::redirect("start", "home");
        assert_eq!(r.status, AwpStatus::Redirect("home".to_string()));
    }

    #[test]
    fn media_stream_status() {
        let r = AwpResponse::media_stream("media/stream", b"stream-data".to_vec());
        assert_eq!(r.status, AwpStatus::MediaStream);
    }

    #[test]
    fn axbw_content_type() {
        let h = AwpResponseHeaders::new("home", &ResourceKind::AxbwPage);
        assert!(h.content_type.contains("application/axbw"));
    }

    #[test]
    fn media_stream_content_type() {
        let h = AwpResponseHeaders::new("media/stream", &ResourceKind::MediaStream);
        assert!(h.content_type.contains("awp-stream"));
    }

    #[test]
    fn headers_include_sovereignty() {
        let h = AwpResponseHeaders::new("home", &ResourceKind::AxbwPage);
        assert!(h.sovereignty.contains("sovereign"));
    }

    #[test]
    fn headers_as_pairs_has_five_entries() {
        let h = AwpResponseHeaders::new("home", &ResourceKind::AxbwPage);
        assert_eq!(h.as_pairs().len(), 5);
    }

    #[test]
    fn cache_control_present_in_headers() {
        let h = AwpResponseHeaders::new("session/token", &ResourceKind::AxbwPage);
        assert!(h.cache_control.contains("no-store"));
    }

    #[test]
    fn static_asset_cache_is_immutable() {
        let h = AwpResponseHeaders::new("assets/logo.png", &ResourceKind::Asset);
        assert!(h.cache_control.contains("immutable"));
    }
}
