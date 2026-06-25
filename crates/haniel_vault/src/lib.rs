// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// HANIEL VAULT — Sovereign Memory Manager and Cache
// HE-2 implementation target

#![forbid(unsafe_code)]

pub mod cache;
pub mod font;
pub mod memory;
pub mod lru;

/// Cache tier
#[derive(Debug, Clone, PartialEq)]
pub enum CacheTier {
    Hot,
    Warm,
    Cold,
}

/// Cache key
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CacheKey {
    pub origin: String,
    pub resource_hash: [u8; 32],
}

impl CacheKey {
    pub fn new(origin: &str, data: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        // Blake3 via axon_crypto at HE-2
        // Placeholder: simple XOR fold for skeleton
        for (i, b) in data.iter().enumerate() {
            hash[i % 32] ^= b;
        }
        Self {
            origin: origin.to_string(),
            resource_hash: hash,
        }
    }
}

/// Memory statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
    pub hot_bytes: usize,
    pub warm_bytes: usize,
    pub texture_bytes: usize,
    pub font_bytes: usize,
    pub total_bytes: usize,
    pub limit_bytes: usize,
}

/// VAULT memory and cache trait
pub trait Vault: Send + Sync {
    fn store(&self, key: CacheKey, data: Vec<u8>) -> Result<(), VaultError>;
    fn get(&self, key: &CacheKey) -> Option<Vec<u8>>;
    fn evict_expired(&self);
    fn stats(&self) -> MemoryStats;
    fn enforce_limit(&self, limit_bytes: usize);
}

/// VAULT error type
#[derive(Debug)]
pub enum VaultError {
    StorageFull,
    KeyNotFound,
    EdisonDbUnreachable,
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_constructs() {
        let key = CacheKey::new("https://example.com", b"test data");
        assert_eq!(key.origin, "https://example.com");
        assert_eq!(key.resource_hash.len(), 32);
    }

    #[test]
    fn cache_tier_variants_exist() {
        assert_ne!(CacheTier::Hot, CacheTier::Cold);
    }

    #[test]
    fn memory_stats_default_zero() {
        let stats = MemoryStats::default();
        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.hot_bytes, 0);
    }

    #[test]
    fn cache_key_different_data_different_hash() {
        let k1 = CacheKey::new("origin", b"data1");
        let k2 = CacheKey::new("origin", b"data2");
        assert_ne!(k1.resource_hash, k2.resource_hash);
    }
}
