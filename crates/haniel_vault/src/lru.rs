// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_vault::lru — Sovereign LRU cache implementation

use std::collections::HashMap;

/// LRU cache node
struct LruNode {
    key:   String,
    value: Vec<u8>,
    prev:  Option<String>,
    next:  Option<String>,
}

/// Sovereign LRU cache — O(1) get and put
pub struct LruCache {
    capacity:  usize,
    used:      usize,
    map:       HashMap<String, LruNode>,
    head:      Option<String>,  // most recently used
    tail:      Option<String>,  // least recently used
}

impl LruCache {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            capacity: capacity_bytes,
            used:     0,
            map:      HashMap::new(),
            head:     None,
            tail:     None,
        }
    }

    /// Get a value — promotes to MRU position
    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        if self.map.contains_key(key) {
            let value = self.map[key].value.clone();
            self.promote(key);
            Some(value)
        } else {
            None
        }
    }

    /// Insert a key-value pair — evicts LRU if over capacity
    pub fn put(&mut self, key: String, value: Vec<u8>) {
        let size = value.len();

        // Remove existing entry if present
        if self.map.contains_key(&key) {
            self.remove_node(&key.clone());
        }

        // Evict LRU entries until we have space
        while self.used + size > self.capacity && self.tail.is_some() {
            let lru_key = self.tail.clone().unwrap();
            self.remove_node(&lru_key);
        }

        // Insert at head (MRU)
        let prev_head = self.head.clone();
        let node = LruNode {
            key:   key.clone(),
            value,
            prev:  None,
            next:  prev_head.clone(),
        };

        if let Some(ref h) = prev_head {
            if let Some(head_node) = self.map.get_mut(h) {
                head_node.prev = Some(key.clone());
            }
        }

        self.head = Some(key.clone());
        if self.tail.is_none() {
            self.tail = Some(key.clone());
        }

        self.used += size;
        self.map.insert(key, node);
    }

    /// Remove a specific key
    pub fn remove(&mut self, key: &str) -> Option<Vec<u8>> {
        if self.map.contains_key(key) {
            let value = self.map[key].value.clone();
            self.remove_node(key);
            Some(value)
        } else {
            None
        }
    }

    /// Current bytes used
    pub fn used_bytes(&self) -> usize {
        self.used
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Promote a key to MRU position
    fn promote(&mut self, key: &str) {
        if self.head.as_deref() == Some(key) {
            return; // already MRU
        }
        let value = self.map[key].value.clone();
        let size  = value.len();
        self.remove_node(key);
        self.used += size; // remove_node decrements, re-add
        let prev_head = self.head.clone();
        let node = LruNode {
            key:  key.to_string(),
            value,
            prev: None,
            next: prev_head.clone(),
        };
        if let Some(ref h) = prev_head {
            if let Some(hn) = self.map.get_mut(h) {
                hn.prev = Some(key.to_string());
            }
        }
        self.head = Some(key.to_string());
        if self.tail.is_none() {
            self.tail = Some(key.to_string());
        }
        self.map.insert(key.to_string(), node);
    }

    /// Remove a node from the doubly-linked list
    fn remove_node(&mut self, key: &str) {
        if let Some(node) = self.map.remove(key) {
            self.used = self.used.saturating_sub(node.value.len());

            match (&node.prev, &node.next) {
                (None, None) => {
                    self.head = None;
                    self.tail = None;
                }
                (None, Some(next)) => {
                    self.head = Some(next.clone());
                    if let Some(n) = self.map.get_mut(next) {
                        n.prev = None;
                    }
                }
                (Some(prev), None) => {
                    self.tail = Some(prev.clone());
                    if let Some(p) = self.map.get_mut(prev) {
                        p.next = None;
                    }
                }
                (Some(prev), Some(next)) => {
                    if let Some(p) = self.map.get_mut(prev) {
                        p.next = Some(next.clone());
                    }
                    if let Some(n) = self.map.get_mut(next) {
                        n.prev = Some(prev.clone());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lru_basic_put_get() {
        let mut cache = LruCache::new(1024);
        cache.put("key1".to_string(), b"hello".to_vec());
        assert_eq!(cache.get("key1"), Some(b"hello".to_vec()));
    }

    #[test]
    fn lru_miss_returns_none() {
        let mut cache = LruCache::new(1024);
        assert_eq!(cache.get("missing"), None);
    }

    #[test]
    fn lru_evicts_lru_when_full() {
        let mut cache = LruCache::new(10);
        cache.put("a".to_string(), b"12345".to_vec()); // 5 bytes
        cache.put("b".to_string(), b"67890".to_vec()); // 5 bytes — now full
        cache.put("c".to_string(), b"xxxxx".to_vec()); // 5 bytes — evicts "a"
        assert_eq!(cache.get("a"), None);   // evicted
        assert!(cache.get("b").is_some());  // still present
        assert!(cache.get("c").is_some());  // just inserted
    }

    #[test]
    fn lru_promotes_on_get() {
        let mut cache = LruCache::new(10);
        cache.put("a".to_string(), b"12345".to_vec());
        cache.put("b".to_string(), b"67890".to_vec());
        // Access "a" — promotes it to MRU
        cache.get("a");
        // Insert "c" — should evict "b" (LRU), not "a"
        cache.put("c".to_string(), b"xxxxx".to_vec());
        assert!(cache.get("a").is_some());  // promoted — survives
        assert_eq!(cache.get("b"), None);   // evicted
    }

    #[test]
    fn lru_remove_entry() {
        let mut cache = LruCache::new(1024);
        cache.put("x".to_string(), b"data".to_vec());
        assert!(cache.remove("x").is_some());
        assert_eq!(cache.get("x"), None);
    }

    #[test]
    fn lru_used_bytes_tracked() {
        let mut cache = LruCache::new(1024);
        assert_eq!(cache.used_bytes(), 0);
        cache.put("k".to_string(), b"hello".to_vec());
        assert_eq!(cache.used_bytes(), 5);
    }

    #[test]
    fn lru_len_tracked() {
        let mut cache = LruCache::new(1024);
        assert_eq!(cache.len(), 0);
        cache.put("a".to_string(), b"1".to_vec());
        cache.put("b".to_string(), b"2".to_vec());
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn lru_overwrite_same_key() {
        let mut cache = LruCache::new(1024);
        cache.put("k".to_string(), b"old".to_vec());
        cache.put("k".to_string(), b"new".to_vec());
        assert_eq!(cache.get("k"), Some(b"new".to_vec()));
        assert_eq!(cache.len(), 1);
    }
}
