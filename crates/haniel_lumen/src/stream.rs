// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::stream — HLS/DASH stream handling
// HE-14: real manifest parsing — .m3u8 (HLS) and .mpd (DASH XML)

use crate::StreamProtocol;

/// Stream manifest — HLS or DASH
#[derive(Debug, Clone)]
pub struct StreamManifest {
    pub protocol:  StreamProtocol,
    pub url:       String,
    pub qualities: Vec<StreamQuality>,
}

/// Quality level in adaptive stream
#[derive(Debug, Clone)]
pub struct StreamQuality {
    pub bitrate_bps: u64,
    pub width:       u32,
    pub height:      u32,
    pub url:         String,
    /// Raw codec string from the manifest, if present
    /// (HLS CODECS="..." attr or DASH codecs="..." attr)
    pub codecs:      Option<String>,
}

/// Manifest parse error
#[derive(Debug, Clone, PartialEq)]
pub enum ManifestParseError {
    Empty,
    NotHls,
    NotDash,
    MalformedVariant(String),
}

impl std::fmt::Display for ManifestParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestParseError::Empty => write!(f, "manifest text is empty"),
            ManifestParseError::NotHls => write!(f, "not a valid HLS manifest (missing #EXTM3U)"),
            ManifestParseError::NotDash => write!(f, "not a valid DASH manifest (missing MPD root)"),
            ManifestParseError::MalformedVariant(s) => write!(f, "malformed variant stream: {}", s),
        }
    }
}

impl StreamManifest {
    pub fn new(protocol: StreamProtocol, url: &str) -> Self {
        Self { protocol, url: url.to_string(), qualities: Vec::new() }
    }

    pub fn add_quality(&mut self, q: StreamQuality) {
        self.qualities.push(q);
    }

    pub fn best_quality(&self) -> Option<&StreamQuality> {
        self.qualities.iter().max_by_key(|q| q.bitrate_bps)
    }

    pub fn lowest_quality(&self) -> Option<&StreamQuality> {
        self.qualities.iter().min_by_key(|q| q.bitrate_bps)
    }

    /// Parse an HLS master playlist (.m3u8 text) into a StreamManifest.
    ///
    /// Recognizes #EXT-X-STREAM-INF lines with BANDWIDTH, RESOLUTION, and
    /// CODECS attributes, followed by a variant playlist URI on the next line.
    pub fn parse_hls(text: &str, base_url: &str) -> Result<Self, ManifestParseError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ManifestParseError::Empty);
        }
        if !text.starts_with("#EXTM3U") {
            return Err(ManifestParseError::NotHls);
        }

        let mut manifest = Self::new(StreamProtocol::Hls, base_url);
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if let Some(attrs) = line.strip_prefix("#EXT-X-STREAM-INF:") {
                let bitrate_bps = parse_hls_attr_u64(attrs, "BANDWIDTH").unwrap_or(0);
                let (width, height) = parse_hls_resolution(attrs).unwrap_or((0, 0));
                let codecs = parse_hls_attr_string(attrs, "CODECS");

                // The variant URI is the next non-empty, non-comment line
                let uri = lines.get(i + 1)
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'));

                match uri {
                    Some(u) => {
                        manifest.add_quality(StreamQuality {
                            bitrate_bps,
                            width,
                            height,
                            url: resolve_relative(base_url, u),
                            codecs,
                        });
                        i += 2;
                        continue;
                    }
                    None => {
                        return Err(ManifestParseError::MalformedVariant(
                            "EXT-X-STREAM-INF missing variant URI".to_string(),
                        ));
                    }
                }
            }
            i += 1;
        }

        Ok(manifest)
    }

    /// Parse a DASH MPD manifest (.mpd XML text) into a StreamManifest.
    ///
    /// This is a minimal attribute-scraping parser — not a full XML parser —
    /// sufficient to extract Representation bandwidth/resolution/codecs for
    /// sovereign ABR selection. Looks for <Representation ...> tags.
    pub fn parse_dash(text: &str, base_url: &str) -> Result<Self, ManifestParseError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(ManifestParseError::Empty);
        }
        if !text.contains("<MPD") {
            return Err(ManifestParseError::NotDash);
        }

        let mut manifest = Self::new(StreamProtocol::Dash, base_url);

        for tag in find_xml_tags(text, "Representation") {
            let bitrate_bps = parse_xml_attr_u64(&tag, "bandwidth").unwrap_or(0);
            let width  = parse_xml_attr_u64(&tag, "width").unwrap_or(0) as u32;
            let height = parse_xml_attr_u64(&tag, "height").unwrap_or(0) as u32;
            let codecs = parse_xml_attr_string(&tag, "codecs");
            let id     = parse_xml_attr_string(&tag, "id").unwrap_or_default();

            manifest.add_quality(StreamQuality {
                bitrate_bps,
                width,
                height,
                url: format!("{}#{}", base_url, id),
                codecs,
            });
        }

        Ok(manifest)
    }
}

// ── HLS attribute parsing helpers ──────────────────────────────────────────────

/// Parse a numeric attribute (e.g. BANDWIDTH=1280000) from an HLS attr-list string
fn parse_hls_attr_u64(attrs: &str, key: &str) -> Option<u64> {
    for part in split_hls_attrs(attrs) {
        if let Some(val) = part.strip_prefix(&format!("{}=", key)) {
            return val.trim_matches('"').parse::<u64>().ok();
        }
    }
    None
}

/// Parse a quoted string attribute (e.g. CODECS="avc1.64001f,mp4a.40.2")
fn parse_hls_attr_string(attrs: &str, key: &str) -> Option<String> {
    for part in split_hls_attrs(attrs) {
        if let Some(val) = part.strip_prefix(&format!("{}=", key)) {
            return Some(val.trim_matches('"').to_string());
        }
    }
    None
}

/// Parse RESOLUTION=1920x1080
fn parse_hls_resolution(attrs: &str) -> Option<(u32, u32)> {
    let raw = parse_hls_attr_string(attrs, "RESOLUTION")?;
    let (w, h) = raw.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

/// Split an HLS attribute list on commas, respecting quoted substrings
/// (CODECS="a,b" must not split inside the quotes)
fn split_hls_attrs(attrs: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in attrs.chars() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
                current.push(c);
            }
            ',' if !in_quotes => {
                parts.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(c),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

/// Resolve a possibly-relative variant URI against the manifest base URL
fn resolve_relative(base_url: &str, uri: &str) -> String {
    if uri.starts_with("http://") || uri.starts_with("https://") || uri.starts_with("awp://") {
        return uri.to_string();
    }
    match base_url.rfind('/') {
        Some(pos) => format!("{}/{}", &base_url[..pos], uri),
        None => uri.to_string(),
    }
}

// ── DASH (minimal XML attribute scraping) helpers ──────────────────────────────

/// Find all `<TagName ...>` opening tags (self-closing or not) and return
/// their full attribute span as a string, for simple attribute extraction.
fn find_xml_tags(text: &str, tag_name: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let open_marker = format!("<{}", tag_name);
    let mut search_from = 0;

    while let Some(start) = text[search_from..].find(&open_marker) {
        let abs_start = search_from + start;
        // Ensure this isn't a longer tag name sharing the prefix (e.g. <RepresentationFoo)
        let after = abs_start + open_marker.len();
        let next_char = text.as_bytes().get(after).copied();
        if !matches!(next_char, Some(b' ') | Some(b'>') | Some(b'/')) {
            search_from = abs_start + open_marker.len();
            continue;
        }

        if let Some(end_rel) = text[abs_start..].find('>') {
            let abs_end = abs_start + end_rel;
            tags.push(text[abs_start..=abs_end].to_string());
            search_from = abs_end + 1;
        } else {
            break;
        }
    }
    tags
}

fn parse_xml_attr_u64(tag: &str, key: &str) -> Option<u64> {
    parse_xml_attr_string(tag, key)?.parse::<u64>().ok()
}

/// Parse a `key="value"` attribute from a tag string.
///
/// Anchors the match so that `key` must be preceded by whitespace or the
/// tag-name boundary — this avoids `width=` falsely matching inside
/// `bandwidth=`, since "bandwidth" ends with the substring "width".
fn parse_xml_attr_string(tag: &str, key: &str) -> Option<String> {
    let marker = format!("{}=\"", key);
    let mut search_from = 0;

    while let Some(rel) = tag[search_from..].find(&marker) {
        let abs = search_from + rel;
        let preceded_by_boundary = abs == 0
            || matches!(tag.as_bytes().get(abs - 1), Some(b' ') | Some(b'\t') | Some(b'\n'));

        if preceded_by_boundary {
            let start = abs + marker.len();
            let end   = tag[start..].find('"')? + start;
            return Some(tag[start..end].to_string());
        }
        search_from = abs + marker.len();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── existing struct-level tests ──

    #[test]
    fn manifest_add_quality() {
        let mut m = StreamManifest::new(StreamProtocol::Hls, "https://example.com/stream.m3u8");
        m.add_quality(StreamQuality { bitrate_bps: 1_000_000, width: 1280, height: 720, url: "720p.m3u8".into(), codecs: None });
        m.add_quality(StreamQuality { bitrate_bps: 3_000_000, width: 1920, height: 1080, url: "1080p.m3u8".into(), codecs: None });
        assert_eq!(m.qualities.len(), 2);
    }

    #[test]
    fn manifest_best_quality() {
        let mut m = StreamManifest::new(StreamProtocol::Dash, "stream.mpd");
        m.add_quality(StreamQuality { bitrate_bps: 500_000,   width: 854,  height: 480,  url: "480p".into(), codecs: None });
        m.add_quality(StreamQuality { bitrate_bps: 2_000_000, width: 1920, height: 1080, url: "1080p".into(), codecs: None });
        let best = m.best_quality().unwrap();
        assert_eq!(best.height, 1080);
    }

    #[test]
    fn manifest_lowest_quality() {
        let mut m = StreamManifest::new(StreamProtocol::Hls, "stream.m3u8");
        m.add_quality(StreamQuality { bitrate_bps: 500_000,   width: 854,  height: 480,  url: "480p".into(), codecs: None });
        m.add_quality(StreamQuality { bitrate_bps: 2_000_000, width: 1920, height: 1080, url: "1080p".into(), codecs: None });
        let lowest = m.lowest_quality().unwrap();
        assert_eq!(lowest.height, 480);
    }

    // ── HLS parsing ──

    const HLS_MASTER: &str = "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-STREAM-INF:BANDWIDTH=800000,RESOLUTION=640x360,CODECS=\"avc1.42001e,mp4a.40.2\"\n360p.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=2500000,RESOLUTION=1280x720,CODECS=\"avc1.4d001f,mp4a.40.2\"\n720p.m3u8\n#EXT-X-STREAM-INF:BANDWIDTH=5000000,RESOLUTION=1920x1080,CODECS=\"av01.0.05M.08\"\n1080p.m3u8\n";

    #[test]
    fn parse_hls_master_basic() {
        let m = StreamManifest::parse_hls(HLS_MASTER, "https://cdn.example.com/master.m3u8").unwrap();
        assert_eq!(m.protocol, StreamProtocol::Hls);
        assert_eq!(m.qualities.len(), 3);
    }

    #[test]
    fn parse_hls_extracts_bandwidth() {
        let m = StreamManifest::parse_hls(HLS_MASTER, "https://cdn.example.com/master.m3u8").unwrap();
        let q = m.qualities.iter().find(|q| q.height == 720).unwrap();
        assert_eq!(q.bitrate_bps, 2_500_000);
    }

    #[test]
    fn parse_hls_extracts_resolution() {
        let m = StreamManifest::parse_hls(HLS_MASTER, "https://cdn.example.com/master.m3u8").unwrap();
        let q = m.qualities.iter().find(|q| q.bitrate_bps == 800_000).unwrap();
        assert_eq!((q.width, q.height), (640, 360));
    }

    #[test]
    fn parse_hls_extracts_codecs() {
        let m = StreamManifest::parse_hls(HLS_MASTER, "https://cdn.example.com/master.m3u8").unwrap();
        let q = m.qualities.iter().find(|q| q.height == 1080).unwrap();
        assert_eq!(q.codecs.as_deref(), Some("av01.0.05M.08"));
    }

    #[test]
    fn parse_hls_resolves_relative_uri() {
        let m = StreamManifest::parse_hls(HLS_MASTER, "https://cdn.example.com/master.m3u8").unwrap();
        let q = m.qualities.iter().find(|q| q.height == 360).unwrap();
        assert_eq!(q.url, "https://cdn.example.com/360p.m3u8");
    }

    #[test]
    fn parse_hls_best_quality_after_parse() {
        let m = StreamManifest::parse_hls(HLS_MASTER, "master.m3u8").unwrap();
        let best = m.best_quality().unwrap();
        assert_eq!(best.height, 1080);
    }

    #[test]
    fn parse_hls_empty_text_errors() {
        assert_eq!(StreamManifest::parse_hls("", "x").unwrap_err(), ManifestParseError::Empty);
    }

    #[test]
    fn parse_hls_missing_header_errors() {
        let bad = "#EXT-X-STREAM-INF:BANDWIDTH=100\nfoo.m3u8\n";
        assert_eq!(StreamManifest::parse_hls(bad, "x").unwrap_err(), ManifestParseError::NotHls);
    }

    #[test]
    fn parse_hls_missing_variant_uri_errors() {
        let bad = "#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=100\n";
        assert!(matches!(
            StreamManifest::parse_hls(bad, "x").unwrap_err(),
            ManifestParseError::MalformedVariant(_)
        ));
    }

    #[test]
    fn parse_hls_no_variants_returns_empty_manifest() {
        let m = StreamManifest::parse_hls("#EXTM3U\n#EXT-X-VERSION:3\n", "x").unwrap();
        assert!(m.qualities.is_empty());
    }

    // ── DASH parsing ──

    const DASH_MPD: &str = "<?xml version=\"1.0\"?>\n<MPD xmlns=\"urn:mpeg:dash:schema:mpd:2011\">\n<Period>\n<AdaptationSet>\n<Representation id=\"360p\" bandwidth=\"800000\" width=\"640\" height=\"360\" codecs=\"avc1.42001e\"/>\n<Representation id=\"720p\" bandwidth=\"2500000\" width=\"1280\" height=\"720\" codecs=\"avc1.4d001f\"/>\n<Representation id=\"1080p\" bandwidth=\"5000000\" width=\"1920\" height=\"1080\" codecs=\"av01.0.05M.08\"/>\n</AdaptationSet>\n</Period>\n</MPD>\n";

    #[test]
    fn parse_dash_mpd_basic() {
        let m = StreamManifest::parse_dash(DASH_MPD, "https://cdn.example.com/stream.mpd").unwrap();
        assert_eq!(m.protocol, StreamProtocol::Dash);
        assert_eq!(m.qualities.len(), 3);
    }

    #[test]
    fn parse_dash_extracts_bandwidth() {
        let m = StreamManifest::parse_dash(DASH_MPD, "x.mpd").unwrap();
        let q = m.qualities.iter().find(|q| q.height == 720).unwrap();
        assert_eq!(q.bitrate_bps, 2_500_000);
    }

    #[test]
    fn parse_dash_extracts_resolution() {
        let m = StreamManifest::parse_dash(DASH_MPD, "x.mpd").unwrap();
        let q = m.qualities.iter().find(|q| q.bitrate_bps == 800_000).unwrap();
        assert_eq!((q.width, q.height), (640, 360));
    }

    #[test]
    fn parse_dash_extracts_codecs() {
        let m = StreamManifest::parse_dash(DASH_MPD, "x.mpd").unwrap();
        let q = m.qualities.iter().find(|q| q.height == 1080).unwrap();
        assert_eq!(q.codecs.as_deref(), Some("av01.0.05M.08"));
    }

    #[test]
    fn parse_dash_best_quality_after_parse() {
        let m = StreamManifest::parse_dash(DASH_MPD, "x.mpd").unwrap();
        let best = m.best_quality().unwrap();
        assert_eq!(best.height, 1080);
    }

    #[test]
    fn parse_dash_empty_text_errors() {
        assert_eq!(StreamManifest::parse_dash("", "x").unwrap_err(), ManifestParseError::Empty);
    }

    #[test]
    fn parse_dash_missing_mpd_root_errors() {
        assert_eq!(StreamManifest::parse_dash("<Other/>", "x").unwrap_err(), ManifestParseError::NotDash);
    }

    #[test]
    fn parse_dash_no_representations_returns_empty() {
        let mpd = "<MPD></MPD>";
        let m = StreamManifest::parse_dash(mpd, "x").unwrap();
        assert!(m.qualities.is_empty());
    }

    // ── helper function unit tests ──

    #[test]
    fn split_hls_attrs_respects_quotes() {
        let attrs = r#"BANDWIDTH=100,CODECS="a,b,c",RESOLUTION=1x1"#;
        let parts = split_hls_attrs(attrs);
        assert_eq!(parts.len(), 3);
        assert!(parts[1].contains("a,b,c"));
    }

    #[test]
    fn resolve_relative_keeps_absolute_url() {
        assert_eq!(resolve_relative("https://a.com/m.m3u8", "https://b.com/v.m3u8"), "https://b.com/v.m3u8");
    }

    #[test]
    fn resolve_relative_joins_relative_path() {
        assert_eq!(resolve_relative("https://a.com/dir/master.m3u8", "v.m3u8"), "https://a.com/dir/v.m3u8");
    }

    #[test]
    fn find_xml_tags_does_not_match_longer_tag_name() {
        let text = "<RepresentationSet><Representation id=\"x\" bandwidth=\"1\"/></RepresentationSet>";
        let tags = find_xml_tags(text, "Representation");
        assert_eq!(tags.len(), 1);
    }
}
