// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL pipeline — sovereign page-load orchestration
// HE-15a: the missing link — fetch a URL, gate it, parse it, lay it out,
// paint it. This is the pipeline that makes HANIEL capable of replacing
// a legacy web-view's content rendering, not just exercising its modules
// individually.

use haniel_canvas::{PixelBuffer, SovereignCanvas};
use haniel_herald::{
    ArpiTier, ContentType, FetchError, FetchResponse, Herald, HeraldFetch, ThreatVerdict,
};
use haniel_prism::{ComputedLayout, Prism, PrismError, SovereignPrism};

/// Outcome of a full sovereign page load: fetch → gate → parse → layout → paint.
#[derive(Debug)]
pub struct PageLoadResult {
    /// The final rasterized pixel buffer, ready to present.
    pub pixels: PixelBuffer,
    /// Computed layout tree, kept for incremental repaint and hit-testing.
    pub layout: ComputedLayout,
    /// Threat verdict from HERALD's STS gate for this URL.
    pub threat_verdict: ThreatVerdict,
    /// ARPi sovereignty tier resolved for this URL's origin.
    pub arpi_tier: ArpiTier,
    /// Content type HERALD classified the response as.
    pub content_type: ContentType,
}

/// Error produced anywhere along the sovereign page-load pipeline.
#[derive(Debug)]
pub enum PageLoadError {
    /// HERALD blocked or failed to fetch the resource.
    Fetch(FetchError),
    /// PRISM failed to parse the fetched body.
    Parse(PrismError),
    /// The fetched content type is not page content (e.g. a bare asset)
    /// and cannot be loaded as a top-level page.
    NotPageContent(ContentType),
}

impl std::fmt::Display for PageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PageLoadError::Fetch(e) => write!(f, "fetch failed: {:?}", e),
            PageLoadError::Parse(e) => write!(f, "parse failed: {:?}", e),
            PageLoadError::NotPageContent(ct) =>
                write!(f, "not loadable as a page: {:?}", ct),
        }
    }
}

/// Which PRISM render pass to use for a page load.
///
/// Rod is the immediate structural skeleton (cheap, used for first paint
/// or low-priority background tabs). Cone is the full-detail pass with
/// real text metrics and asset placement (used for the active/foreground
/// page). Sovereign Render Budget (SRB) tooling in `haniel_prism::srb`
/// is the intended future driver for this choice; HE-15a exposes the
/// choice explicitly rather than deciding it implicitly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PassQuality {
    Rod,
    Cone,
}

/// Sovereign page loader — the orchestration HANIEL needs to act as a
/// legacy web-view replacement: it owns one HERALD fetcher, one PRISM
/// parser/layout engine, and one CANVAS rasterizer, and chains them.
pub struct PageLoader {
    herald: HeraldFetch,
    prism: SovereignPrism,
    canvas: SovereignCanvas,
}

impl PageLoader {
    /// Construct a page loader targeting a viewport of `width` x `height`
    /// device pixels.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            herald: HeraldFetch::new(),
            prism: SovereignPrism::new(),
            canvas: SovereignCanvas::new(width, height),
        }
    }

    /// Resize the loader's viewport / canvas. Subsequent `load` calls use
    /// the new dimensions; layout from a previous load is not retroactively
    /// rescaled.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas.resize(width, height);
    }

    /// Current viewport width in device pixels.
    pub fn width(&self) -> u32 {
        self.canvas.width
    }

    /// Current viewport height in device pixels.
    pub fn height(&self) -> u32 {
        self.canvas.height
    }

    /// Load a URL end-to-end: HERALD fetch + threat gate, PRISM parse +
    /// layout, CANVAS paint. Returns the final pixel buffer plus the
    /// layout tree and HERALD's verdicts, so a caller (e.g. Onyxia's
    /// content surface) can present the pixels and drive UI state (the
    /// ARPi trust bar, threat warnings) from the same call.
    ///
    /// `quality` selects PRISM's rod (fast, structural) or cone (full
    /// detail) pass. HE-15a leaves the choice to the caller rather than
    /// inferring it, since "first paint vs. settled paint" is a UI-layer
    /// policy decision, not a rendering-pipeline one.
    pub fn load(&self, uri: &str, quality: PassQuality) -> Result<PageLoadResult, PageLoadError> {
        let response: FetchResponse =
            Herald::fetch(&self.herald, uri).map_err(PageLoadError::Fetch)?;

        let tree = match response.content_type {
            ContentType::Axbw => {
                // HE-15a seam note: HERALD's current registries (both the
                // HE-3 AwpRegistry stub and HE-13 haniel_awp::AwpRegistry)
                // emit human-readable "<axbw>...</axbw>" text markup, not
                // the real binary AXBW container format PRISM's
                // AxbwParser expects (4-byte "AXBW" magic + version
                // header). Until a real AXBW-emitting tool exists, content
                // tagged Axbw is attempted as binary first and falls back
                // to the HTML tokenizer — which tolerates the markup as
                // ordinary (if semantically opaque) tags — rather than
                // failing the whole page load outright.
                let body_str = std::str::from_utf8(&response.body).unwrap_or("");
                match self.prism.parse_axbw(&response.body) {
                    Ok(t) => t,
                    Err(_) => self
                        .prism
                        .parse_html(body_str)
                        .map_err(PageLoadError::Parse)?,
                }
            }
            ContentType::HtmlSubset => {
                let html = std::str::from_utf8(&response.body).unwrap_or("");
                self.prism.parse_html(html).map_err(PageLoadError::Parse)?
            }
            ContentType::Asset(_) => {
                return Err(PageLoadError::NotPageContent(response.content_type));
            }
        };

        let vw = self.canvas.width as f32;
        let vh = self.canvas.height as f32;

        let layout = match quality {
            PassQuality::Rod => self.prism.rod_pass(&tree, vw, vh).map_err(PageLoadError::Parse)?,
            PassQuality::Cone => self.prism.cone_pass(&tree, vw, vh).map_err(PageLoadError::Parse)?,
        };

        let is_html = matches!(response.content_type, ContentType::HtmlSubset);
        let pixels = match quality {
            PassQuality::Rod => self.canvas.paint_rod(&layout),
            PassQuality::Cone => self.canvas.paint_cone(&layout, is_html, false),
        };

        Ok(PageLoadResult {
            pixels,
            layout,
            threat_verdict: response.threat_verdict,
            arpi_tier: response.arpi_tier,
            content_type: response.content_type,
        })
    }

    /// Convenience: load with the cone (full-detail) pass, the common case
    /// for a foreground page load.
    pub fn load_full(&self, uri: &str) -> Result<PageLoadResult, PageLoadError> {
        self.load(uri, PassQuality::Cone)
    }

    /// Convenience: load with the rod (structural skeleton) pass, for fast
    /// first paint before the cone pass settles.
    pub fn load_skeleton(&self, uri: &str) -> Result<PageLoadResult, PageLoadError> {
        self.load(uri, PassQuality::Rod)
    }

    /// Run HERALD's threat gate on a URL without fetching or rendering it.
    /// Useful for pre-navigation warnings (e.g. hovering a link) where the
    /// caller wants a verdict before committing to a full page load.
    pub fn precheck(&self, uri: &str) -> ThreatVerdict {
        Herald::threat_gate(&self.herald, uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loader() -> PageLoader {
        PageLoader::new(1280, 720)
    }

    #[test]
    fn loader_constructs_with_viewport() {
        let l = loader();
        assert_eq!(l.width(), 1280);
        assert_eq!(l.height(), 720);
    }

    #[test]
    fn loader_resize_updates_viewport() {
        let mut l = loader();
        l.resize(1920, 1080);
        assert_eq!(l.width(), 1920);
        assert_eq!(l.height(), 1080);
    }

    #[test]
    fn load_full_sovereign_awp_page_succeeds() {
        let l = loader();
        let result = l.load_full("awp://aegis");
        assert!(result.is_ok());
    }

    #[test]
    fn load_full_sovereign_page_arpi_tier_sovereign() {
        let l = loader();
        let result = l.load_full("awp://aegis").unwrap();
        assert_eq!(result.arpi_tier, ArpiTier::Sovereign);
    }

    #[test]
    fn load_full_produces_nonzero_pixel_buffer() {
        let l = loader();
        let result = l.load_full("awp://aegis").unwrap();
        assert_eq!(result.pixels.width, 1280);
        assert_eq!(result.pixels.height, 720);
        assert!(!result.pixels.is_transparent());
    }

    #[test]
    fn load_skeleton_produces_pixel_buffer() {
        let l = loader();
        let result = l.load_skeleton("awp://aegis").unwrap();
        assert!(!result.pixels.is_transparent());
    }

    #[test]
    fn load_full_layout_has_root_id() {
        let l = loader();
        let result = l.load_full("awp://aegis").unwrap();
        assert!(!result.layout.id.is_empty());
    }

    #[test]
    fn load_unknown_awp_path_succeeds_with_not_found_body() {
        // HERALD's AwpRegistry::serve falls back to a synthetic
        // "not found" AXBW body for unregistered paths rather than
        // erroring — fetch() itself only errors on threat-gate blocks,
        // network failure, or malformed URIs. An unknown but otherwise
        // well-formed awp:// path is therefore a successful load whose
        // body happens to say "not found".
        let l = loader();
        let result = l.load_full("awp://does-not-exist-at-all").unwrap();
        assert_eq!(result.arpi_tier, ArpiTier::Sovereign);
    }

    #[test]
    fn load_tracker_domain_blocked() {
        let l = loader();
        let result = l.load_full("https://doubleclick.net/");
        assert!(matches!(result, Err(PageLoadError::Fetch(_))));
    }

    #[test]
    fn precheck_clean_url_is_clean() {
        let l = loader();
        let verdict = l.precheck("https://example.com/");
        assert_eq!(verdict, ThreatVerdict::Clean);
    }

    #[test]
    fn precheck_tracker_url_blocked() {
        let l = loader();
        let verdict = l.precheck("https://doubleclick.net/");
        assert!(matches!(verdict, ThreatVerdict::Blocked(_)));
    }

    #[test]
    fn precheck_does_not_require_network() {
        // precheck must be usable purely from a URL string, no fetch involved —
        // confirmed by it returning instantly for a domain HERALD has no
        // route to fetch (no awp:// registry entry, no real HTTPS network
        // in this test environment).
        let l = loader();
        let verdict = l.precheck("https://totally-unregistered-domain-xyz.example/");
        assert_eq!(verdict, ThreatVerdict::Clean);
    }

    #[test]
    fn page_load_error_display_fetch() {
        let e = PageLoadError::Fetch(FetchError::Timeout);
        assert!(e.to_string().contains("fetch failed"));
    }

    #[test]
    fn page_load_error_display_not_page_content() {
        let e = PageLoadError::NotPageContent(ContentType::Asset(
            haniel_herald::AssetKind::Image,
        ));
        assert!(e.to_string().contains("not loadable as a page"));
    }

    #[test]
    fn quality_variants_distinct() {
        assert_ne!(PassQuality::Rod, PassQuality::Cone);
    }

    #[test]
    fn load_full_and_skeleton_agree_on_root_id() {
        let l = loader();
        let full = l.load_full("awp://aegis").unwrap();
        let skel = l.load_skeleton("awp://aegis").unwrap();
        assert_eq!(full.layout.id, skel.layout.id);
    }
}
