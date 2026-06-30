// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_lumen::demux — Sovereign segment list extraction
// HE-14: HLS media playlist + DASH SegmentList parsing
//
// Scope note: this module parses adaptive-stream *segment lists* (the
// ordered sequence of media segment URIs a player walks through), not
// raw MP4/WebM container boxes. Byte-level container demuxing depends on
// axon_media primitives that do not exist yet — that work is tracked
// separately and is not blocking HANIEL Stage 3.

/// A single media segment in a sequential playback list
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// Resolved (or relative) URI of the segment
    pub url: String,
    /// Segment duration in seconds (HLS #EXTINF, or DASH derived duration)
    pub duration_secs: f64,
    /// Sequence index, starting at 0
    pub sequence: u64,
}

/// An ordered, sovereign-fetched segment list for a single quality variant
#[derive(Debug, Clone, Default)]
pub struct SegmentList {
    pub segments: Vec<Segment>,
    /// True if this is a live stream (HLS #EXT-X-ENDLIST absent)
    pub is_live: bool,
}

/// Segment list parse error
#[derive(Debug, Clone, PartialEq)]
pub enum SegmentParseError {
    Empty,
    NotHlsMediaPlaylist,
    NotDashSegmentList,
    MalformedSegment(String),
}

impl std::fmt::Display for SegmentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SegmentParseError::Empty => write!(f, "segment list text is empty"),
            SegmentParseError::NotHlsMediaPlaylist =>
                write!(f, "not a valid HLS media playlist (missing #EXTM3U)"),
            SegmentParseError::NotDashSegmentList =>
                write!(f, "not a valid DASH SegmentList (missing SegmentList tag)"),
            SegmentParseError::MalformedSegment(s) =>
                write!(f, "malformed segment entry: {}", s),
        }
    }
}

impl SegmentList {
    pub fn new() -> Self {
        Self { segments: Vec::new(), is_live: false }
    }

    pub fn total_duration_secs(&self) -> f64 {
        self.segments.iter().map(|s| s.duration_secs).sum()
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    /// Get the segment for a given playback position in seconds
    pub fn segment_at(&self, position_secs: f64) -> Option<&Segment> {
        let mut elapsed = 0.0;
        for seg in &self.segments {
            elapsed += seg.duration_secs;
            if position_secs < elapsed {
                return Some(seg);
            }
        }
        self.segments.last()
    }

    /// Parse an HLS media playlist (variant .m3u8 — distinct from the
    /// master playlist parsed in `stream::parse_hls`). Recognizes
    /// #EXTINF:<duration>, lines and the following segment URI, plus
    /// #EXT-X-ENDLIST to detect VOD vs live.
    pub fn parse_hls_media_playlist(text: &str, base_url: &str) -> Result<Self, SegmentParseError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(SegmentParseError::Empty);
        }
        if !text.starts_with("#EXTM3U") {
            return Err(SegmentParseError::NotHlsMediaPlaylist);
        }

        let mut list = Self::new();
        list.is_live = !text.contains("#EXT-X-ENDLIST");

        let lines: Vec<&str> = text.lines().collect();
        let mut sequence = 0u64;
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            if let Some(rest) = line.strip_prefix("#EXTINF:") {
                // #EXTINF:9.009,  (optional trailing title after comma)
                let duration_str = rest.split(',').next().unwrap_or("").trim();
                let duration_secs = duration_str.parse::<f64>().map_err(|_| {
                    SegmentParseError::MalformedSegment(format!("bad #EXTINF duration: {}", rest))
                })?;

                let uri = lines.get(i + 1)
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !l.starts_with('#'));

                match uri {
                    Some(u) => {
                        list.segments.push(Segment {
                            url: resolve_relative(base_url, u),
                            duration_secs,
                            sequence,
                        });
                        sequence += 1;
                        i += 2;
                        continue;
                    }
                    None => {
                        return Err(SegmentParseError::MalformedSegment(
                            "#EXTINF missing following segment URI".to_string(),
                        ));
                    }
                }
            }
            i += 1;
        }

        Ok(list)
    }

    /// Parse a DASH SegmentList block (minimal attribute scraping — looks
    /// for <SegmentURL media="..."/> entries inside a <SegmentList> tag,
    /// with a shared duration from the SegmentList's `duration` attribute
    /// expressed in timescale units, converted to seconds via `timescale`).
    pub fn parse_dash_segment_list(text: &str, base_url: &str) -> Result<Self, SegmentParseError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(SegmentParseError::Empty);
        }
        if !text.contains("<SegmentList") {
            return Err(SegmentParseError::NotDashSegmentList);
        }

        let mut list = Self::new();
        list.is_live = false; // SegmentList is a VOD construct in this minimal model

        let seg_list_tag = find_first_tag(text, "SegmentList")
            .ok_or(SegmentParseError::NotDashSegmentList)?;
        let timescale = parse_attr_u64(&seg_list_tag, "timescale").unwrap_or(1).max(1);
        let duration_units = parse_attr_u64(&seg_list_tag, "duration").unwrap_or(0);
        let duration_secs = duration_units as f64 / timescale as f64;

        for (sequence, tag) in find_all_tags(text, "SegmentURL").into_iter().enumerate() {
            let media = parse_attr_str(&tag, "media").ok_or_else(|| {
                SegmentParseError::MalformedSegment("SegmentURL missing media attr".to_string())
            })?;
            list.segments.push(Segment {
                url: resolve_relative(base_url, &media),
                duration_secs,
                sequence: sequence as u64,
            });
        }

        Ok(list)
    }
}

// ── helpers ─────────────────────────────────────────────────────────────────

fn resolve_relative(base_url: &str, uri: &str) -> String {
    if uri.starts_with("http://") || uri.starts_with("https://") || uri.starts_with("awp://") {
        return uri.to_string();
    }
    match base_url.rfind('/') {
        Some(pos) => format!("{}/{}", &base_url[..pos], uri),
        None => uri.to_string(),
    }
}

fn find_first_tag(text: &str, tag_name: &str) -> Option<String> {
    find_all_tags(text, tag_name).into_iter().next()
}

fn find_all_tags(text: &str, tag_name: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let open_marker = format!("<{}", tag_name);
    let mut search_from = 0;

    while let Some(start) = text[search_from..].find(&open_marker) {
        let abs_start = search_from + start;
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

fn parse_attr_u64(tag: &str, key: &str) -> Option<u64> {
    parse_attr_str(tag, key)?.parse::<u64>().ok()
}

/// Parse a `key="value"` attribute from a tag string, anchored so that
/// `key` must be preceded by whitespace or the tag-name boundary — this
/// avoids one attribute name falsely matching as a suffix of another
/// (e.g. a hypothetical `key` matching inside `superkey=`).
fn parse_attr_str(tag: &str, key: &str) -> Option<String> {
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

    // ── SegmentList basics ──

    #[test]
    fn segment_list_starts_empty() {
        let list = SegmentList::new();
        assert_eq!(list.segment_count(), 0);
        assert_eq!(list.total_duration_secs(), 0.0);
    }

    #[test]
    fn total_duration_sums_segments() {
        let mut list = SegmentList::new();
        list.segments.push(Segment { url: "a".into(), duration_secs: 9.0, sequence: 0 });
        list.segments.push(Segment { url: "b".into(), duration_secs: 9.5, sequence: 1 });
        assert!((list.total_duration_secs() - 18.5).abs() < 0.001);
    }

    #[test]
    fn segment_at_finds_correct_segment() {
        let mut list = SegmentList::new();
        list.segments.push(Segment { url: "a".into(), duration_secs: 10.0, sequence: 0 });
        list.segments.push(Segment { url: "b".into(), duration_secs: 10.0, sequence: 1 });
        list.segments.push(Segment { url: "c".into(), duration_secs: 10.0, sequence: 2 });

        assert_eq!(list.segment_at(5.0).unwrap().url, "a");
        assert_eq!(list.segment_at(15.0).unwrap().url, "b");
        assert_eq!(list.segment_at(25.0).unwrap().url, "c");
    }

    #[test]
    fn segment_at_past_end_returns_last() {
        let mut list = SegmentList::new();
        list.segments.push(Segment { url: "a".into(), duration_secs: 10.0, sequence: 0 });
        assert_eq!(list.segment_at(999.0).unwrap().url, "a");
    }

    #[test]
    fn segment_at_empty_list_returns_none() {
        let list = SegmentList::new();
        assert!(list.segment_at(5.0).is_none());
    }

    // ── HLS media playlist parsing ──

    const HLS_MEDIA_VOD: &str = "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:10\n#EXTINF:9.009,\nseg0.ts\n#EXTINF:9.009,\nseg1.ts\n#EXTINF:8.500,\nseg2.ts\n#EXT-X-ENDLIST\n";

    const HLS_MEDIA_LIVE: &str = "#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:10\n#EXTINF:9.009,\nseg100.ts\n#EXTINF:9.009,\nseg101.ts\n";

    #[test]
    fn parse_hls_media_vod_segment_count() {
        let list = SegmentList::parse_hls_media_playlist(HLS_MEDIA_VOD, "https://cdn.example.com/720p.m3u8").unwrap();
        assert_eq!(list.segment_count(), 3);
    }

    #[test]
    fn parse_hls_media_vod_is_not_live() {
        let list = SegmentList::parse_hls_media_playlist(HLS_MEDIA_VOD, "x.m3u8").unwrap();
        assert!(!list.is_live);
    }

    #[test]
    fn parse_hls_media_live_is_live() {
        let list = SegmentList::parse_hls_media_playlist(HLS_MEDIA_LIVE, "x.m3u8").unwrap();
        assert!(list.is_live);
    }

    #[test]
    fn parse_hls_media_extracts_durations() {
        let list = SegmentList::parse_hls_media_playlist(HLS_MEDIA_VOD, "x.m3u8").unwrap();
        assert!((list.segments[0].duration_secs - 9.009).abs() < 0.001);
        assert!((list.segments[2].duration_secs - 8.500).abs() < 0.001);
    }

    #[test]
    fn parse_hls_media_resolves_segment_urls() {
        let list = SegmentList::parse_hls_media_playlist(HLS_MEDIA_VOD, "https://cdn.example.com/720p.m3u8").unwrap();
        assert_eq!(list.segments[0].url, "https://cdn.example.com/seg0.ts");
    }

    #[test]
    fn parse_hls_media_sequence_numbers_increment() {
        let list = SegmentList::parse_hls_media_playlist(HLS_MEDIA_VOD, "x.m3u8").unwrap();
        assert_eq!(list.segments[0].sequence, 0);
        assert_eq!(list.segments[1].sequence, 1);
        assert_eq!(list.segments[2].sequence, 2);
    }

    #[test]
    fn parse_hls_media_empty_errors() {
        assert_eq!(SegmentList::parse_hls_media_playlist("", "x").unwrap_err(), SegmentParseError::Empty);
    }

    #[test]
    fn parse_hls_media_missing_header_errors() {
        let bad = "#EXTINF:9,\nseg0.ts\n";
        assert_eq!(
            SegmentList::parse_hls_media_playlist(bad, "x").unwrap_err(),
            SegmentParseError::NotHlsMediaPlaylist
        );
    }

    #[test]
    fn parse_hls_media_missing_uri_errors() {
        let bad = "#EXTM3U\n#EXTINF:9,\n";
        assert!(matches!(
            SegmentList::parse_hls_media_playlist(bad, "x").unwrap_err(),
            SegmentParseError::MalformedSegment(_)
        ));
    }

    #[test]
    fn parse_hls_media_bad_duration_errors() {
        let bad = "#EXTM3U\n#EXTINF:notanumber,\nseg0.ts\n";
        assert!(matches!(
            SegmentList::parse_hls_media_playlist(bad, "x").unwrap_err(),
            SegmentParseError::MalformedSegment(_)
        ));
    }

    #[test]
    fn parse_hls_media_total_duration() {
        let list = SegmentList::parse_hls_media_playlist(HLS_MEDIA_VOD, "x.m3u8").unwrap();
        assert!((list.total_duration_secs() - 26.518).abs() < 0.001);
    }

    // ── DASH SegmentList parsing ──

    const DASH_SEGLIST: &str = "<SegmentList timescale=\"1\" duration=\"10\">\n<SegmentURL media=\"seg0.m4s\"/>\n<SegmentURL media=\"seg1.m4s\"/>\n<SegmentURL media=\"seg2.m4s\"/>\n</SegmentList>\n";

    #[test]
    fn parse_dash_seglist_count() {
        let list = SegmentList::parse_dash_segment_list(DASH_SEGLIST, "https://cdn.example.com/stream.mpd").unwrap();
        assert_eq!(list.segment_count(), 3);
    }

    #[test]
    fn parse_dash_seglist_duration_from_timescale() {
        let list = SegmentList::parse_dash_segment_list(DASH_SEGLIST, "x.mpd").unwrap();
        assert!((list.segments[0].duration_secs - 10.0).abs() < 0.001);
    }

    #[test]
    fn parse_dash_seglist_resolves_urls() {
        let list = SegmentList::parse_dash_segment_list(DASH_SEGLIST, "https://cdn.example.com/stream.mpd").unwrap();
        assert_eq!(list.segments[0].url, "https://cdn.example.com/seg0.m4s");
    }

    #[test]
    fn parse_dash_seglist_sequence_increments() {
        let list = SegmentList::parse_dash_segment_list(DASH_SEGLIST, "x.mpd").unwrap();
        assert_eq!(list.segments[1].sequence, 1);
    }

    #[test]
    fn parse_dash_seglist_empty_errors() {
        assert_eq!(SegmentList::parse_dash_segment_list("", "x").unwrap_err(), SegmentParseError::Empty);
    }

    #[test]
    fn parse_dash_seglist_missing_tag_errors() {
        assert_eq!(
            SegmentList::parse_dash_segment_list("<Other/>", "x").unwrap_err(),
            SegmentParseError::NotDashSegmentList
        );
    }

    #[test]
    fn parse_dash_seglist_timescale_conversion() {
        let mpd = "<SegmentList timescale=\"90000\" duration=\"900000\"><SegmentURL media=\"a.m4s\"/></SegmentList>";
        let list = SegmentList::parse_dash_segment_list(mpd, "x.mpd").unwrap();
        assert!((list.segments[0].duration_secs - 10.0).abs() < 0.001);
    }

    #[test]
    fn parse_dash_seglist_default_timescale_one() {
        let mpd = "<SegmentList duration=\"5\"><SegmentURL media=\"a.m4s\"/></SegmentList>";
        let list = SegmentList::parse_dash_segment_list(mpd, "x.mpd").unwrap();
        assert!((list.segments[0].duration_secs - 5.0).abs() < 0.001);
    }
}
