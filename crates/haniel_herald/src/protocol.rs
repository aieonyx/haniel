// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_herald::protocol — Protocol detection

use crate::{Protocol, ContentType, AssetKind};

pub fn detect(uri: &str) -> Protocol {
    if uri.starts_with("awp://")   { Protocol::Awp   }
    else if uri.starts_with("https://") { Protocol::Https }
    else                           { Protocol::Http  }
}

pub fn detect_content_type(uri: &str, ct_header: Option<&str>) -> ContentType {
    if let Some(ct) = ct_header {
        if ct.contains("application/axbw")        { return ContentType::Axbw; }
        if ct.contains("text/html")               { return ContentType::HtmlSubset; }
        if ct.contains("image/")                  { return ContentType::Asset(AssetKind::Image); }
        if ct.contains("font/")                   { return ContentType::Asset(AssetKind::Font); }
        if ct.contains("javascript")              { return ContentType::Asset(AssetKind::Script); }
        if ct.contains("text/css")                { return ContentType::Asset(AssetKind::StyleModule); }
    }
    let lower = uri.to_lowercase();
    if lower.ends_with(".axbw")                   { ContentType::Axbw }
    else if lower.ends_with(".html") || lower.ends_with(".htm") { ContentType::HtmlSubset }
    else if lower.ends_with(".png") || lower.ends_with(".jpg")
         || lower.ends_with(".webp") || lower.ends_with(".svg") { ContentType::Asset(AssetKind::Image) }
    else if lower.ends_with(".woff2") || lower.ends_with(".ttf") { ContentType::Asset(AssetKind::Font) }
    else if lower.ends_with(".js") || lower.ends_with(".wasm")  { ContentType::Asset(AssetKind::Script) }
    else if lower.ends_with(".css")               { ContentType::Asset(AssetKind::StyleModule) }
    else                                          { ContentType::HtmlSubset }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_awp()   { assert_eq!(detect("awp://aegis"),       Protocol::Awp);   }
    #[test]
    fn detect_https() { assert_eq!(detect("https://example.com"), Protocol::Https); }
    #[test]
    fn detect_http()  { assert_eq!(detect("http://example.com"),  Protocol::Http);  }
    #[test]
    fn detect_unknown_is_http() { assert_eq!(detect("ftp://x.com"), Protocol::Http); }

    #[test]
    fn ct_axbw_from_header() {
        assert_eq!(detect_content_type("", Some("application/axbw")), ContentType::Axbw);
    }
    #[test]
    fn ct_html_from_header() {
        assert_eq!(detect_content_type("", Some("text/html; charset=utf-8")), ContentType::HtmlSubset);
    }
    #[test]
    fn ct_image_from_ext() {
        assert_eq!(detect_content_type("https://cdn.com/img.png", None), ContentType::Asset(AssetKind::Image));
    }
    #[test]
    fn ct_font_from_ext() {
        assert_eq!(detect_content_type("https://cdn.com/f.woff2", None), ContentType::Asset(AssetKind::Font));
    }
    #[test]
    fn ct_script_from_ext() {
        assert_eq!(detect_content_type("https://cdn.com/app.js", None), ContentType::Asset(AssetKind::Script));
    }
}
