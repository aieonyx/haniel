// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_awp::youtube — Sovereign YouTube compatibility layer
//
// Pipeline:
//   1. Detect — is this a YouTube URL? (youtube.com / youtu.be / youtube-nocookie.com)
//   2. Strip  — remove all tracking parameters (utm_*, si, feature, list, index, etc.)
//   3. Extract — pull the clean video ID
//   4. Route  — produce a YoutubeRequest → LUMEN adaptive stream
//
// Sovereignty principle: YouTube content is allowed, YouTube surveillance is not.
// Tracking parameters are stripped before any URL leaves HANIEL.
// DRM notice is preserved from LUMEN HE-10: sovereign playback, no Widevine.

use crate::AwpError;

/// Known YouTube hostnames
const YT_HOSTS: &[&str] = &[
    "youtube.com",
    "youtu.be",
    "youtube-nocookie.com",
    "music.youtube.com",
];

/// Query parameters that are tracking / non-essential and must be stripped
const TRACKING_PARAMS: &[&str] = &[
    // Tracking
    "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content",
    "si",          // Session identity token (YouTube internal tracking)
    "feature",     // YouTube UI feature tracking
    "app",         // App source tracking
    "src",         // Referral source
    "ref",         // Referral
    "fbclid",      // Facebook click ID
    "gclid",       // Google click ID
    "dclid",       // DoubleClick click ID
    // Playlist / autoplay context (not part of sovereign video identity)
    "list",        // Playlist ID
    "index",       // Position in playlist
    "start_radio", // YouTube radio autoplay
    "ab_channel",  // A/B channel tracking
    // Shorts / discovery
    "pp",          // Shorts parameter
    "rco",         // Recommendation context
];

/// Preferred resolution hint for LUMEN ABR
#[derive(Debug, Clone, PartialEq, Default)]
pub enum YoutubeResolution {
    R144p,
    R360p,
    R480p,
    R720p,
    R1080p,
    R1440p,
    R2160p,
    #[default]
    Auto,
}

/// A cleaned, sovereignty-ready YouTube request for LUMEN
#[derive(Debug, Clone)]
pub struct YoutubeRequest {
    /// Clean 11-character video ID
    pub video_id: String,
    /// Timestamp offset in seconds (from `t` or `start` param), if present
    pub start_secs: Option<u32>,
    /// Preferred resolution hint
    pub preferred_resolution: YoutubeResolution,
    /// Original URL (for audit / logging only — never forwarded with tracking)
    pub original_url: String,
    /// Number of tracking parameters that were stripped
    pub stripped_param_count: u32,
}

/// YouTube sovereign compatibility engine
pub struct YoutubeCompat {
    _private: (),
}

impl YoutubeCompat {
    pub fn new() -> Self { Self { _private: () } }

    /// Check whether a host string is a YouTube domain
    pub fn is_youtube_url(&self, host: &str) -> bool {
        let h = host.trim_start_matches("www.").to_lowercase();
        YT_HOSTS.iter().any(|&known| h == known || h.ends_with(&format!(".{}", known)))
    }

    /// Full pipeline: parse → strip → extract → return YoutubeRequest
    pub fn extract_request(&self, url: &str) -> Result<YoutubeRequest, AwpError> {
        let url = url.trim();

        // Parse scheme + host + path + query
        let (host, path, query_str) = self.split_url(url)?;
        let host_clean = host.trim_start_matches("www.").to_lowercase();

        // Verify it's actually YouTube
        if !self.is_youtube_url(&host_clean) {
            return Err(AwpError::YoutubeExtractionFailed(
                format!("not a YouTube host: {}", host_clean),
            ));
        }

        // Extract video ID
        let video_id = self.extract_video_id(&host_clean, path, query_str)?;

        // Validate video ID format (11 alphanumeric chars + - _)
        if !Self::is_valid_video_id(&video_id) {
            return Err(AwpError::YoutubeExtractionFailed(
                format!("invalid video ID format: {}", video_id),
            ));
        }

        // Parse remaining non-tracking params
        let (start_secs, stripped_count) = self.parse_allowed_params(query_str);

        Ok(YoutubeRequest {
            video_id,
            start_secs,
            preferred_resolution: YoutubeResolution::Auto,
            original_url: url.to_string(),
            stripped_param_count: stripped_count,
        })
    }

    /// Strip all tracking parameters, return clean query string (only allowed params remain)
    pub fn strip_tracking(&self, query: &str) -> (String, u32) {
        let mut kept = Vec::new();
        let mut stripped = 0u32;

        for part in query.split('&') {
            if part.is_empty() { continue; }
            let key = if let Some(pos) = part.find('=') {
                &part[..pos]
            } else {
                part
            };
            if TRACKING_PARAMS.contains(&key) {
                stripped += 1;
            } else {
                kept.push(part.to_string());
            }
        }
        (kept.join("&"), stripped)
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn split_url<'a>(&self, url: &'a str) -> Result<(&'a str, &'a str, &'a str), AwpError> {
        // Strip scheme
        let rest = if let Some(pos) = url.find("://") {
            &url[pos + 3..]
        } else {
            return Err(AwpError::YoutubeExtractionFailed(
                "missing scheme".to_string(),
            ));
        };

        // Split host / path+query
        let (host, path_query) = if let Some(pos) = rest.find('/') {
            (&rest[..pos], &rest[pos + 1..])
        } else {
            // No path — host only
            return Ok((rest, "", ""));
        };

        // Split path / query
        let (path, query) = if let Some(pos) = path_query.find('?') {
            (&path_query[..pos], &path_query[pos + 1..])
        } else {
            (path_query, "")
        };

        Ok((host, path, query))
    }

    fn extract_video_id(
        &self,
        host: &str,
        path: &str,
        query: &str,
    ) -> Result<String, AwpError> {
        // youtu.be/<id>
        if host == "youtu.be" {
            let id = path.trim_matches('/');
            if id.is_empty() {
                return Err(AwpError::YoutubeExtractionFailed(
                    "youtu.be URL missing video ID in path".to_string(),
                ));
            }
            return Ok(id.to_string());
        }

        // youtube.com/shorts/<id>
        if let Some(stripped) = path.strip_prefix("shorts/") {
            let id = stripped.trim_matches('/');
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }

        // youtube.com/embed/<id>
        if let Some(stripped) = path.strip_prefix("embed/") {
            let id = stripped.trim_matches('/');
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }

        // youtube.com/v/<id>
        if let Some(stripped) = path.strip_prefix("v/") {
            let id = stripped.trim_matches('/');
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }

        // youtube.com/live/<id>
        if let Some(stripped) = path.strip_prefix("live/") {
            let id = stripped.trim_matches('/');
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }

        // youtube.com/watch?v=<id>  (most common)
        for part in query.split('&') {
            if let Some(id) = part.strip_prefix("v=") {
                if !id.is_empty() {
                    return Ok(id.to_string());
                }
            }
        }

        Err(AwpError::YoutubeExtractionFailed(
            format!("cannot extract video ID from path='{}' query='{}'", path, query),
        ))
    }

    fn parse_allowed_params(&self, query: &str) -> (Option<u32>, u32) {
        let mut start_secs: Option<u32> = None;
        let mut stripped = 0u32;

        for part in query.split('&') {
            if part.is_empty() { continue; }

            let (key, val) = if let Some(pos) = part.find('=') {
                (&part[..pos], &part[pos + 1..])
            } else {
                continue;
            };

            if TRACKING_PARAMS.contains(&key) {
                stripped += 1;
                continue;
            }

            // `t` or `start` = timestamp in seconds
            if key == "t" || key == "start" {
                if let Ok(secs) = val.parse::<u32>() {
                    start_secs = Some(secs);
                }
            }
        }
        (start_secs, stripped)
    }

    fn is_valid_video_id(id: &str) -> bool {
        if id.len() != 11 { return false; }
        id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }
}

impl Default for YoutubeCompat {
    fn default() -> Self { Self::new() }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn yt() -> YoutubeCompat { YoutubeCompat::new() }

    // ── is_youtube_url ──

    #[test]
    fn detects_youtube_com() {
        assert!(yt().is_youtube_url("youtube.com"));
    }

    #[test]
    fn detects_www_youtube_com() {
        assert!(yt().is_youtube_url("www.youtube.com"));
    }

    #[test]
    fn detects_youtu_be() {
        assert!(yt().is_youtube_url("youtu.be"));
    }

    #[test]
    fn detects_nocookie() {
        assert!(yt().is_youtube_url("youtube-nocookie.com"));
    }

    #[test]
    fn detects_music_youtube() {
        assert!(yt().is_youtube_url("music.youtube.com"));
    }

    #[test]
    fn rejects_non_youtube() {
        assert!(!yt().is_youtube_url("example.com"));
        assert!(!yt().is_youtube_url("vimeo.com"));
        assert!(!yt().is_youtube_url("aieonyx.io"));
    }

    // ── strip_tracking ──

    #[test]
    fn strip_si_parameter() {
        let (clean, n) = yt().strip_tracking("v=dQw4w9WgXcQ&si=abc123");
        assert_eq!(n, 1);
        assert!(!clean.contains("si="));
        assert!(clean.contains("v=dQw4w9WgXcQ"));
    }

    #[test]
    fn strip_multiple_tracking_params() {
        let q = "v=dQw4w9WgXcQ&si=x&utm_source=share&feature=youtu.be&list=PL123";
        let (clean, n) = yt().strip_tracking(q);
        assert_eq!(n, 4); // si, utm_source, feature, list
        assert!(clean.contains("v=dQw4w9WgXcQ"));
        assert!(!clean.contains("si="));
        assert!(!clean.contains("utm_source"));
        assert!(!clean.contains("feature="));
        assert!(!clean.contains("list="));
    }

    #[test]
    fn strip_preserves_t_timestamp() {
        let (clean, _) = yt().strip_tracking("v=dQw4w9WgXcQ&t=42&si=abc");
        assert!(clean.contains("t=42"));
    }

    #[test]
    fn strip_empty_query() {
        let (clean, n) = yt().strip_tracking("");
        assert_eq!(n, 0);
        assert!(clean.is_empty());
    }

    #[test]
    fn strip_all_tracking_returns_empty() {
        let q = "si=x&utm_source=y&feature=z";
        let (clean, n) = yt().strip_tracking(q);
        assert_eq!(n, 3);
        assert!(clean.is_empty());
    }

    // ── extract_request — standard watch URL ──

    #[test]
    fn extract_standard_watch_url() {
        let req = yt()
            .extract_request("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
            .unwrap();
        assert_eq!(req.video_id, "dQw4w9WgXcQ");
    }

    #[test]
    fn extract_watch_url_strips_tracking() {
        let req = yt()
            .extract_request("https://www.youtube.com/watch?v=dQw4w9WgXcQ&si=abc&utm_source=x")
            .unwrap();
        assert_eq!(req.video_id, "dQw4w9WgXcQ");
        assert_eq!(req.stripped_param_count, 2);
    }

    #[test]
    fn extract_short_url() {
        let req = yt()
            .extract_request("https://youtu.be/dQw4w9WgXcQ")
            .unwrap();
        assert_eq!(req.video_id, "dQw4w9WgXcQ");
    }

    #[test]
    fn extract_short_url_with_tracking() {
        let req = yt()
            .extract_request("https://youtu.be/dQw4w9WgXcQ?si=abc123&feature=share")
            .unwrap();
        assert_eq!(req.video_id, "dQw4w9WgXcQ");
        assert_eq!(req.stripped_param_count, 2);
    }

    #[test]
    fn extract_embed_url() {
        let req = yt()
            .extract_request("https://www.youtube.com/embed/dQw4w9WgXcQ")
            .unwrap();
        assert_eq!(req.video_id, "dQw4w9WgXcQ");
    }

    #[test]
    fn extract_shorts_url() {
        let req = yt()
            .extract_request("https://www.youtube.com/shorts/dQw4w9WgXcQ")
            .unwrap();
        assert_eq!(req.video_id, "dQw4w9WgXcQ");
    }

    #[test]
    fn extract_live_url() {
        let req = yt()
            .extract_request("https://www.youtube.com/live/dQw4w9WgXcQ")
            .unwrap();
        assert_eq!(req.video_id, "dQw4w9WgXcQ");
    }

    #[test]
    fn extract_nocookie_url() {
        let req = yt()
            .extract_request("https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ")
            .unwrap();
        assert_eq!(req.video_id, "dQw4w9WgXcQ");
    }

    // ── timestamp extraction ──

    #[test]
    fn extract_timestamp_t_param() {
        let req = yt()
            .extract_request("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=73")
            .unwrap();
        assert_eq!(req.start_secs, Some(73));
    }

    #[test]
    fn extract_no_timestamp() {
        let req = yt()
            .extract_request("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
            .unwrap();
        assert!(req.start_secs.is_none());
    }

    // ── error cases ──

    #[test]
    fn error_non_youtube_host() {
        assert!(yt().extract_request("https://vimeo.com/123456789").is_err());
    }

    #[test]
    fn error_missing_video_id() {
        assert!(yt()
            .extract_request("https://www.youtube.com/watch?list=PLabc")
            .is_err());
    }

    #[test]
    fn error_empty_url() {
        assert!(yt().extract_request("").is_err());
    }

    // ── validity check ──

    #[test]
    fn valid_video_id_11_chars() {
        assert!(YoutubeCompat::is_valid_video_id("dQw4w9WgXcQ"));
    }

    #[test]
    fn invalid_video_id_too_short() {
        assert!(!YoutubeCompat::is_valid_video_id("short"));
    }

    #[test]
    fn invalid_video_id_too_long() {
        assert!(!YoutubeCompat::is_valid_video_id("toolongvideoidstring"));
    }

    #[test]
    fn valid_video_id_with_dash_underscore() {
        // IDs with - and _ are common on YouTube
        assert!(YoutubeCompat::is_valid_video_id("a-b_cDeFgHi"));
    }

    // ── sovereignty asserts ──

    #[test]
    fn original_url_preserved_in_request() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&si=abc";
        let req = yt().extract_request(url).unwrap();
        assert_eq!(req.original_url, url);
    }

    #[test]
    fn default_resolution_is_auto() {
        let req = yt()
            .extract_request("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
            .unwrap();
        assert_eq!(req.preferred_resolution, YoutubeResolution::Auto);
    }
}
