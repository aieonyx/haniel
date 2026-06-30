// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL AWP — Sovereign AWP Protocol Engine
// HE-13: AWP engine + YouTube compatibility
//
// AWP = AIEONYX Web Protocol — the sovereign URI scheme
// Sovereign Cache Doctrine: each response carries cache semantics from day one
// YouTube compat: detect → strip tracking → extract ID → route to LUMEN adaptive stream

#![forbid(unsafe_code)]

pub mod cache;
pub mod registry;
pub mod response;
pub mod router;
pub mod youtube;

pub use cache::{AwpCacheDirective, AwpCachePolicy};
pub use registry::{AwpRegistry, AwpResource, ResourceKind};
pub use response::{AwpResponse, AwpResponseHeaders};
pub use router::{AwpRouter, AwpRoute};
pub use youtube::{YoutubeCompat, YoutubeRequest, YoutubeResolution};

/// AWP engine version
pub const AWP_VERSION: &str = "1.0";

/// AWP URI scheme prefix
pub const AWP_SCHEME: &str = "awp://";

/// AXON-Client header value for AWP responses
pub const AWP_CLIENT_HEADER: &str =
    "AXON-Client/1.0 HANIEL/0.1 AWP/1.0 AIEONYX/sovereign";

/// AWP engine error
#[derive(Debug, Clone, PartialEq)]
pub enum AwpError {
    /// Resource not found at this path
    NotFound(String),
    /// Request rejected by sovereignty policy
    PolicyRejected(String),
    /// Route parse failed
    MalformedUri(String),
    /// YouTube extraction failed
    YoutubeExtractionFailed(String),
}

impl std::fmt::Display for AwpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AwpError::NotFound(p)                  => write!(f, "AWP resource not found: {}", p),
            AwpError::PolicyRejected(r)            => write!(f, "AWP policy rejected: {}", r),
            AwpError::MalformedUri(u)              => write!(f, "Malformed AWP URI: {}", u),
            AwpError::YoutubeExtractionFailed(e)   => write!(f, "YouTube extraction failed: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn awp_scheme_is_correct() {
        assert_eq!(AWP_SCHEME, "awp://");
    }

    #[test]
    fn awp_version_set() {
        assert!(!AWP_VERSION.is_empty());
    }

    #[test]
    fn client_header_contains_sovereign() {
        assert!(AWP_CLIENT_HEADER.contains("sovereign"));
        assert!(AWP_CLIENT_HEADER.contains("HANIEL"));
    }

    #[test]
    fn awp_error_display_not_found() {
        let e = AwpError::NotFound("aegis/unknown".to_string());
        assert!(e.to_string().contains("not found"));
    }

    #[test]
    fn awp_error_display_policy_rejected() {
        let e = AwpError::PolicyRejected("tracker payload".to_string());
        assert!(e.to_string().contains("policy rejected"));
    }
}
