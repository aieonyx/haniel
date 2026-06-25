// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_vault::font — Font entry cache (wired at HE-8)

/// Cached font entry
#[derive(Debug, Clone)]
pub struct FontEntry {
    pub family:  String,
    pub weight:  u16,
    pub data:    Vec<u8>,   // raw TTF/OTF bytes
}

/// Font cache — keyed by family + weight
pub struct FontCache {
    entries: std::collections::HashMap<String, FontEntry>,
}

impl FontCache {
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }

    pub fn store(&mut self, family: &str, weight: u16, data: Vec<u8>) {
        let key = format!("{}:{}", family, weight);
        self.entries.insert(key, FontEntry {
            family: family.to_string(),
            weight,
            data,
        });
    }

    pub fn get(&self, family: &str, weight: u16) -> Option<&FontEntry> {
        let key = format!("{}:{}", family, weight);
        self.entries.get(&key)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for FontCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_cache_store_get() {
        let mut fc = FontCache::new();
        fc.store("SovereignSans", 400, b"ttf-data".to_vec());
        let entry = fc.get("SovereignSans", 400).unwrap();
        assert_eq!(entry.family, "SovereignSans");
        assert_eq!(entry.weight, 400);
        assert_eq!(entry.data, b"ttf-data");
    }

    #[test]
    fn font_cache_miss() {
        let fc = FontCache::new();
        assert!(fc.get("MissingFont", 400).is_none());
    }

    #[test]
    fn font_cache_len() {
        let mut fc = FontCache::new();
        assert_eq!(fc.len(), 0);
        fc.store("FontA", 400, vec![1, 2, 3]);
        fc.store("FontB", 700, vec![4, 5, 6]);
        assert_eq!(fc.len(), 2);
    }
}
