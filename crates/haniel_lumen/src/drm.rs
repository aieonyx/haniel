// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::drm — Sovereign DRM handling
// No silent failures — Sovereignty Notice shown for DRM content

use crate::DrmMode;

/// DRM verdict — what LUMEN does with DRM content
#[derive(Debug, Clone, PartialEq)]
pub enum DrmVerdict {
    /// Open content — full playback
    Open,
    /// Sovereign key — AIEONYX Ed25519 protected content
    SovereignKeyValid,
    /// Platform DRM (Widevine etc) — show Sovereignty Notice
    PlatformDrmBlocked(String),
    /// Hostile DRM — blocked with explanation
    HostileDrmBlocked(String),
}

/// Sovereignty Notice — shown instead of silent DRM failure
#[derive(Debug, Clone)]
pub struct SovereigntyNotice {
    pub title:       String,
    pub reason:      String,
    pub drm_name:    String,
    pub alternative: String,
}

impl SovereigntyNotice {
    /// Notice for Widevine/platform DRM content
    pub fn platform_drm(drm_name: &str) -> Self {
        Self {
            title:    "Content Protected by Platform DRM".to_string(),
            reason:   format!(
                "This content uses {} which requires authorization from a third-party                 corporation. HANIEL cannot play this content without surrendering your                 identity to that corporation.",
                drm_name
            ),
            drm_name: drm_name.to_string(),
            alternative: "Look for the same content on sovereign or open platforms.".to_string(),
        }
    }

    /// Notice for hostile DRM (surveillance required)
    pub fn hostile_drm(reason: &str) -> Self {
        Self {
            title:    "Content Blocked — Hostile DRM".to_string(),
            reason:   format!(
                "This content requires surveillance telemetry as part of its DRM.                 HANIEL has blocked it to protect your sovereignty. Reason: {}",
                reason
            ),
            drm_name: "Hostile DRM".to_string(),
            alternative: "AIEONYX cannot help you if you give the key away.".to_string(),
        }
    }

    /// Render notice as AXBW-compatible text
    pub fn to_text(&self) -> String {
        format!(
            "[SOVEREIGNTY NOTICE]
{}

{}

Alternative: {}",
            self.title, self.reason, self.alternative
        )
    }
}

/// DRM resolver — determines how to handle DRM for a media source
pub struct DrmResolver;

impl DrmResolver {
    pub fn new() -> Self { Self }

    /// Resolve DRM verdict for a given DRM mode
    pub fn resolve(&self, drm: &DrmMode) -> DrmVerdict {
        match drm {
            DrmMode::None => DrmVerdict::Open,
            DrmMode::SovereignKey(_) => DrmVerdict::SovereignKeyValid,
            DrmMode::CompatibilityShim => {
                DrmVerdict::PlatformDrmBlocked("Platform DRM".to_string())
            }
            DrmMode::Blocked => {
                DrmVerdict::HostileDrmBlocked(
                    "Content requires surveillance telemetry".to_string()
                )
            }
        }
    }

    /// Get sovereignty notice if DRM blocks playback
    pub fn notice(&self, drm: &DrmMode) -> Option<SovereigntyNotice> {
        match self.resolve(drm) {
            DrmVerdict::PlatformDrmBlocked(name) => {
                Some(SovereigntyNotice::platform_drm(&name))
            }
            DrmVerdict::HostileDrmBlocked(reason) => {
                Some(SovereigntyNotice::hostile_drm(&reason))
            }
            _ => None,
        }
    }

    /// Check if content can play
    pub fn can_play(&self, drm: &DrmMode) -> bool {
        matches!(
            self.resolve(drm),
            DrmVerdict::Open | DrmVerdict::SovereignKeyValid
        )
    }
}

impl Default for DrmResolver {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolver() -> DrmResolver { DrmResolver::new() }

    #[test]
    fn open_content_can_play() {
        assert!(resolver().can_play(&DrmMode::None));
    }

    #[test]
    fn sovereign_key_can_play() {
        assert!(resolver().can_play(&DrmMode::SovereignKey(vec![1,2,3])));
    }

    #[test]
    fn platform_drm_blocked() {
        assert!(!resolver().can_play(&DrmMode::CompatibilityShim));
    }

    #[test]
    fn hostile_drm_blocked() {
        assert!(!resolver().can_play(&DrmMode::Blocked));
    }

    #[test]
    fn platform_drm_gives_notice() {
        let notice = resolver().notice(&DrmMode::CompatibilityShim);
        assert!(notice.is_some());
        let n = notice.unwrap();
        assert!(!n.title.is_empty());
        assert!(!n.reason.is_empty());
        assert!(!n.alternative.is_empty());
    }

    #[test]
    fn hostile_drm_gives_notice_with_key_phrase() {
        let notice = resolver().notice(&DrmMode::Blocked).unwrap();
        // Sovereign Consent Doctrine key phrase
        assert!(notice.alternative.contains("AIEONYX cannot help you if you give the key away"));
    }

    #[test]
    fn open_content_no_notice() {
        assert!(resolver().notice(&DrmMode::None).is_none());
    }

    #[test]
    fn sovereignty_notice_to_text() {
        let notice = SovereigntyNotice::platform_drm("Widevine");
        let text   = notice.to_text();
        assert!(text.contains("SOVEREIGNTY NOTICE"));
        assert!(text.contains("Widevine"));
    }

    #[test]
    fn drm_verdict_open() {
        assert_eq!(resolver().resolve(&DrmMode::None), DrmVerdict::Open);
    }

    #[test]
    fn drm_verdict_platform_blocked() {
        assert!(matches!(
            resolver().resolve(&DrmMode::CompatibilityShim),
            DrmVerdict::PlatformDrmBlocked(_)
        ));
    }
}
