// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_vault::cache — Hot and Warm tier sovereign cache

use std::sync::Mutex;
use crate::lru::LruCache;
use crate::memory::MemoryTracker;
use crate::{CacheKey, VaultError};

/// Hot tier — in-process RAM LRU cache
pub struct HotCache {
    lru:     Mutex<LruCache>,
    tracker: MemoryTracker,
}

impl HotCache {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            lru:     Mutex::new(LruCache::new(capacity_bytes)),
            tracker: MemoryTracker::new(capacity_bytes),
        }
    }

    pub fn get(&self, key: &CacheKey) -> Option<Vec<u8>> {
        let k = key.as_str();
        self.lru.lock().unwrap().get(&k)
    }

    pub fn put(&self, key: CacheKey, data: Vec<u8>) -> Result<(), VaultError> {
        let size = data.len();
        let k    = key.as_str().to_string();
        self.lru.lock().unwrap().put(k, data);
        self.tracker.add_hot(size);
        Ok(())
    }

    pub fn remove(&self, key: &CacheKey) -> bool {
        let k = key.as_str().to_string();
        let removed = self.lru.lock().unwrap().remove(&k);
        if let Some(ref data) = removed {
            self.tracker.sub_hot(data.len());
        }
        removed.is_some()
    }

    pub fn used_bytes(&self) -> usize {
        self.lru.lock().unwrap().used_bytes()
    }

    pub fn len(&self) -> usize {
        self.lru.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.lru.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(origin: &str, data: &[u8]) -> CacheKey {
        CacheKey::new(origin, data)
    }

    #[test]
    fn hot_cache_put_get() {
        let cache = HotCache::new(1024 * 1024);
        let key   = make_key("https://example.com", b"resource.css");
        cache.put(key.clone(), b"body { color: red; }".to_vec()).unwrap();
        let result = cache.get(&key);
        assert_eq!(result, Some(b"body { color: red; }".to_vec()));
    }

    #[test]
    fn hot_cache_miss() {
        let cache = HotCache::new(1024 * 1024);
        let key   = make_key("https://example.com", b"missing.js");
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn hot_cache_remove() {
        let cache = HotCache::new(1024 * 1024);
        let key   = make_key("https://example.com", b"data");
        cache.put(key.clone(), b"value".to_vec()).unwrap();
        assert!(cache.remove(&key));
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn hot_cache_tracks_size() {
        let cache = HotCache::new(1024 * 1024);
        let key   = make_key("origin", b"resource");
        cache.put(key, b"12345".to_vec()).unwrap();
        assert_eq!(cache.used_bytes(), 5);
    }
}
