// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL VAULT — Sovereign Memory Manager and Cache
// HE-2 implementation

#![forbid(unsafe_code)]

pub mod cache;
pub mod font;
pub mod memory;
pub mod lru;

pub use memory::MemorySnapshot;
pub use font::FontCache;

use std::sync::{Arc, Mutex};
use cache::HotCache;
use memory::MemoryTracker;

/// Cache tier
#[derive(Debug, Clone, PartialEq)]
pub enum CacheTier {
    Hot,   // in-process RAM — current page assets
    Warm,  // EdisonDB Noise tier — recent pages (HE-2 stub, wired at HE-3)
    Cold,  // EdisonDB Personal tier — trusted site assets
}

/// Cache key — origin + Blake3 hash of resource identity
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub origin:        String,
    pub resource_hash: [u8; 32],
}

impl CacheKey {
    /// Construct a cache key.
    /// Blake3 via axon_crypto wired at HE-3.
    /// Skeleton uses XOR fold as placeholder.
    pub fn new(origin: &str, resource_id: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        for (i, b) in resource_id.iter().enumerate() {
            hash[i % 32] ^= b;
        }
        // Mix in origin bytes
        for (i, b) in origin.bytes().enumerate() {
            hash[(i + 16) % 32] ^= b;
        }
        Self {
            origin: origin.to_string(),
            resource_hash: hash,
        }
    }

    /// String representation for LRU key
    pub fn as_str(&self) -> String {
        format!("{}:{}", self.origin,
            self.resource_hash.iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>())
    }
}

/// Memory statistics snapshot
#[derive(Debug, Default, Clone)]
pub struct MemoryStats {
    pub hot_bytes:     usize,
    pub warm_bytes:    usize,
    pub texture_bytes: usize,
    pub font_bytes:    usize,
    pub total_bytes:   usize,
    pub limit_bytes:   usize,
}

/// Vault error type
#[derive(Debug)]
pub enum VaultError {
    StorageFull,
    KeyNotFound,
    EdisonDbUnreachable,
    SerializationError(String),
}

/// HANIEL VAULT — sovereign memory and cache manager
pub struct SovereignVault {
    hot:     Arc<HotCache>,
    tracker: MemoryTracker,
    fonts:   Mutex<FontCache>,
}

impl SovereignVault {
    /// Create a new VAULT with given memory limit
    pub fn new(limit_bytes: usize) -> Self {
        Self {
            hot:     Arc::new(HotCache::new(limit_bytes)),
            tracker: MemoryTracker::new(limit_bytes),
            fonts:   Mutex::new(FontCache::new()),
        }
    }

    /// Store a resource in the Hot cache tier
    pub fn store(&self, key: CacheKey, data: Vec<u8>) -> Result<(), VaultError> {
        self.hot.put(key, data)
    }

    /// Retrieve a resource — Hot tier only (Warm at HE-3)
    pub fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        self.hot.get(key)
    }

    /// Remove a resource from cache
    pub fn remove(&self, key: &CacheKey) -> bool {
        self.hot.remove(key)
    }

    /// Store a font in font cache
    pub fn store_font(&self, family: &str, weight: u16, data: Vec<u8>) {
        let size = data.len();
        self.fonts.lock().unwrap().store(family, weight, data);
        self.tracker.add_font(size);
    }

    /// Get a font from font cache
    pub fn get_font(&self, family: &str, weight: u16) -> Option<font::FontEntry> {
        self.fonts.lock().unwrap().get(family, weight).cloned()
    }

    /// Current memory statistics
    pub fn stats(&self) -> MemoryStats {
        let snap = self.tracker.snapshot();
        MemoryStats {
            hot_bytes:     self.hot.used_bytes(),
            warm_bytes:    snap.warm_bytes,
            texture_bytes: snap.texture_bytes,
            font_bytes:    snap.font_bytes,
            total_bytes:   self.hot.used_bytes()
                + snap.warm_bytes
                + snap.texture_bytes
                + snap.font_bytes,
            limit_bytes:   snap.limit_bytes,
        }
    }

    /// Enforce memory limit — evict from Hot tier
    /// Full multi-tier eviction wired at HE-8
    pub fn enforce_limit(&self, limit_bytes: usize) {
        self.tracker.set_limit(limit_bytes);
        // Hot tier LRU self-evicts — capacity enforced at put() time
        // Warm/Cold tier eviction: EdisonDB TTL enforcement (HE-3)
    }

    /// Number of entries in hot cache
    pub fn hot_len(&self) -> usize {
        self.hot.len()
    }
}

/// VAULT sovereign trait
pub trait Vault: Send + Sync {
    fn store(&self, key: CacheKey, data: Vec<u8>) -> Result<(), VaultError>;
    fn get(&self, key: &CacheKey) -> Option<Vec<u8>>;
    fn stats(&self) -> MemoryStats;
    fn enforce_limit(&self, limit_bytes: usize);
}

impl Vault for SovereignVault {
    fn store(&self, key: CacheKey, data: Vec<u8>) -> Result<(), VaultError> {
        self.store(key, data)
    }
    fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        self.get(key)
    }
    fn stats(&self) -> MemoryStats {
        self.stats()
    }
    fn enforce_limit(&self, limit_bytes: usize) {
        self.enforce_limit(limit_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vault() -> SovereignVault {
        SovereignVault::new(1024 * 1024) // 1MB test limit
    }

    #[test]
    fn vault_store_and_get() {
        let v   = vault();
        let key = CacheKey::new("https://example.com", b"style.css");
        v.store(key.clone(), b"body{}".to_vec()).unwrap();
        assert_eq!(v.get(&key), Some(b"body{}".to_vec()));
    }

    #[test]
    fn vault_miss_returns_none() {
        let v   = vault();
        let key = CacheKey::new("https://example.com", b"missing");
        assert_eq!(v.get(&key), None);
    }

    #[test]
    fn vault_remove_entry() {
        let v   = vault();
        let key = CacheKey::new("origin", b"res");
        v.store(key.clone(), b"data".to_vec()).unwrap();
        assert!(v.remove(&key));
        assert_eq!(v.get(&key), None);
    }

    #[test]
    fn vault_stats_hot_bytes() {
        let v   = vault();
        let key = CacheKey::new("origin", b"res");
        v.store(key, b"hello".to_vec()).unwrap();
        let stats = v.stats();
        assert_eq!(stats.hot_bytes, 5);
    }

    #[test]
    fn vault_font_store_get() {
        let v = vault();
        v.store_font("SovereignSans", 400, b"ttf".to_vec());
        let entry = v.get_font("SovereignSans", 400).unwrap();
        assert_eq!(entry.family, "SovereignSans");
        assert_eq!(entry.weight, 400);
    }

    #[test]
    fn vault_font_miss() {
        let v = vault();
        assert!(v.get_font("MissingFont", 400).is_none());
    }

    #[test]
    fn cache_key_different_resources_differ() {
        let k1 = CacheKey::new("origin", b"resource_a");
        let k2 = CacheKey::new("origin", b"resource_b");
        assert_ne!(k1.resource_hash, k2.resource_hash);
    }

    #[test]
    fn cache_key_same_resource_same_hash() {
        let k1 = CacheKey::new("origin", b"resource");
        let k2 = CacheKey::new("origin", b"resource");
        assert_eq!(k1.resource_hash, k2.resource_hash);
    }

    #[test]
    fn cache_key_different_origins_differ() {
        let k1 = CacheKey::new("https://a.com", b"res");
        let k2 = CacheKey::new("https://b.com", b"res");
        assert_ne!(k1, k2);
    }

    #[test]
    fn vault_enforce_limit_does_not_panic() {
        let v = vault();
        v.enforce_limit(512 * 1024);
        let stats = v.stats();
        assert_eq!(stats.limit_bytes, 512 * 1024);
    }

    #[test]
    fn vault_hot_len_tracked() {
        let v   = vault();
        assert_eq!(v.hot_len(), 0);
        v.store(CacheKey::new("o", b"a"), b"1".to_vec()).unwrap();
        v.store(CacheKey::new("o", b"b"), b"2".to_vec()).unwrap();
        assert_eq!(v.hot_len(), 2);
    }
}
