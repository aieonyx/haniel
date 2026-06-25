// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_vault::memory — Sovereign memory statistics tracker

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Live memory statistics — atomic, lock-free
#[derive(Debug, Clone)]
pub struct MemoryTracker {
    inner: Arc<MemoryInner>,
}

#[derive(Debug, Default)]
struct MemoryInner {
    hot_bytes:     AtomicUsize,
    warm_bytes:    AtomicUsize,
    texture_bytes: AtomicUsize,
    font_bytes:    AtomicUsize,
    limit_bytes:   AtomicUsize,
}

impl MemoryTracker {
    pub fn new(limit_bytes: usize) -> Self {
        let inner = Arc::new(MemoryInner::default());
        inner.limit_bytes.store(limit_bytes, Ordering::Relaxed);
        Self { inner }
    }

    pub fn add_hot(&self, bytes: usize) {
        self.inner.hot_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn sub_hot(&self, bytes: usize) {
        self.inner.hot_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    pub fn add_warm(&self, bytes: usize) {
        self.inner.warm_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn sub_warm(&self, bytes: usize) {
        self.inner.warm_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    pub fn add_texture(&self, bytes: usize) {
        self.inner.texture_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn sub_texture(&self, bytes: usize) {
        self.inner.texture_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    pub fn add_font(&self, bytes: usize) {
        self.inner.font_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn sub_font(&self, bytes: usize) {
        self.inner.font_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    pub fn set_limit(&self, bytes: usize) {
        self.inner.limit_bytes.store(bytes, Ordering::Relaxed);
    }

    pub fn hot_bytes(&self) -> usize {
        self.inner.hot_bytes.load(Ordering::Relaxed)
    }

    pub fn warm_bytes(&self) -> usize {
        self.inner.warm_bytes.load(Ordering::Relaxed)
    }

    pub fn texture_bytes(&self) -> usize {
        self.inner.texture_bytes.load(Ordering::Relaxed)
    }

    pub fn font_bytes(&self) -> usize {
        self.inner.font_bytes.load(Ordering::Relaxed)
    }

    pub fn limit_bytes(&self) -> usize {
        self.inner.limit_bytes.load(Ordering::Relaxed)
    }

    pub fn total_bytes(&self) -> usize {
        self.hot_bytes()
            + self.warm_bytes()
            + self.texture_bytes()
            + self.font_bytes()
    }

    pub fn is_over_limit(&self) -> bool {
        self.total_bytes() > self.limit_bytes()
    }

    pub fn snapshot(&self) -> MemorySnapshot {
        MemorySnapshot {
            hot_bytes:     self.hot_bytes(),
            warm_bytes:    self.warm_bytes(),
            texture_bytes: self.texture_bytes(),
            font_bytes:    self.font_bytes(),
            total_bytes:   self.total_bytes(),
            limit_bytes:   self.limit_bytes(),
        }
    }
}

/// Point-in-time memory snapshot
#[derive(Debug, Clone, Default)]
pub struct MemorySnapshot {
    pub hot_bytes:     usize,
    pub warm_bytes:    usize,
    pub texture_bytes: usize,
    pub font_bytes:    usize,
    pub total_bytes:   usize,
    pub limit_bytes:   usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracker_starts_at_zero() {
        let t = MemoryTracker::new(1024 * 1024);
        assert_eq!(t.total_bytes(), 0);
        assert_eq!(t.hot_bytes(), 0);
    }

    #[test]
    fn tracker_add_hot() {
        let t = MemoryTracker::new(1024 * 1024);
        t.add_hot(512);
        assert_eq!(t.hot_bytes(), 512);
        assert_eq!(t.total_bytes(), 512);
    }

    #[test]
    fn tracker_not_over_limit_when_empty() {
        let t = MemoryTracker::new(1024);
        assert!(!t.is_over_limit());
    }

    #[test]
    fn tracker_over_limit_detected() {
        let t = MemoryTracker::new(100);
        t.add_hot(200);
        assert!(t.is_over_limit());
    }

    #[test]
    fn tracker_snapshot_correct() {
        let t = MemoryTracker::new(4096);
        t.add_hot(100);
        t.add_warm(200);
        t.add_texture(300);
        t.add_font(50);
        let snap = t.snapshot();
        assert_eq!(snap.hot_bytes, 100);
        assert_eq!(snap.warm_bytes, 200);
        assert_eq!(snap.texture_bytes, 300);
        assert_eq!(snap.font_bytes, 50);
        assert_eq!(snap.total_bytes, 650);
        assert_eq!(snap.limit_bytes, 4096);
    }

    #[test]
    fn tracker_sub_hot() {
        let t = MemoryTracker::new(1024);
        t.add_hot(200);
        t.sub_hot(100);
        assert_eq!(t.hot_bytes(), 100);
    }
}
