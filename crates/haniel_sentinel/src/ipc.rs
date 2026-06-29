// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
// haniel_sentinel::ipc — Inter-Process Communication channel
// Stub: software queue now, seL4 shared memory at HE-15

use crate::{SentinelError, SentinelMessage};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// IPC channel direction
#[derive(Debug, Clone, PartialEq)]
pub enum IpcDirection {
    TrustedToRenderer,
    RendererToTrusted,
    TrustedToScript,
    ScriptToTrusted,
}

/// IPC channel — message passing between PDs
pub struct IpcChannel {
    pub direction: IpcDirection,
    queue:         Arc<Mutex<VecDeque<SentinelMessage>>>,
    capacity:      usize,
}

impl IpcChannel {
    pub fn new(direction: IpcDirection, capacity: usize) -> Self {
        Self {
            direction,
            queue:    Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    /// Send a message — non-blocking
    pub fn send(&self, msg: SentinelMessage) -> Result<(), SentinelError> {
        let mut q = self.queue.lock().unwrap();
        if q.len() >= self.capacity {
            return Err(SentinelError::IpcFull);
        }
        q.push_back(msg);
        Ok(())
    }

    /// Receive next message — non-blocking
    pub fn recv(&self) -> Option<SentinelMessage> {
        self.queue.lock().unwrap().pop_front()
    }

    /// Peek at next message without consuming
    pub fn peek(&self) -> bool {
        !self.queue.lock().unwrap().is_empty()
    }

    /// Number of messages waiting
    pub fn pending(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// Drain all messages
    pub fn drain(&self) -> Vec<SentinelMessage> {
        self.queue.lock().unwrap().drain(..).collect()
    }

    /// Clone the underlying queue handle (for dual-endpoint access)
    pub fn clone_handle(&self) -> Self {
        Self {
            direction: self.direction.clone(),
            queue:     Arc::clone(&self.queue),
            capacity:  self.capacity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SentinelMessage;

    fn channel() -> IpcChannel {
        IpcChannel::new(IpcDirection::TrustedToRenderer, 16)
    }

    #[test]
    fn send_and_recv() {
        let ch = channel();
        ch.send(SentinelMessage::RenderFrame { frame_id: 1 }).unwrap();
        let msg = ch.recv().unwrap();
        assert!(matches!(msg, SentinelMessage::RenderFrame { frame_id: 1 }));
    }

    #[test]
    fn recv_empty_returns_none() {
        let ch = channel();
        assert!(ch.recv().is_none());
    }

    #[test]
    fn channel_capacity_enforced() {
        let ch = IpcChannel::new(IpcDirection::TrustedToRenderer, 2);
        ch.send(SentinelMessage::RenderFrame { frame_id: 1 }).unwrap();
        ch.send(SentinelMessage::RenderFrame { frame_id: 2 }).unwrap();
        let result = ch.send(SentinelMessage::RenderFrame { frame_id: 3 });
        assert!(matches!(result, Err(SentinelError::IpcFull)));
    }

    #[test]
    fn pending_count() {
        let ch = channel();
        assert_eq!(ch.pending(), 0);
        ch.send(SentinelMessage::Ping).unwrap();
        ch.send(SentinelMessage::Ping).unwrap();
        assert_eq!(ch.pending(), 2);
    }

    #[test]
    fn peek_true_when_messages_waiting() {
        let ch = channel();
        assert!(!ch.peek());
        ch.send(SentinelMessage::Ping).unwrap();
        assert!(ch.peek());
    }

    #[test]
    fn drain_clears_all() {
        let ch = channel();
        ch.send(SentinelMessage::Ping).unwrap();
        ch.send(SentinelMessage::Ping).unwrap();
        let msgs = ch.drain();
        assert_eq!(msgs.len(), 2);
        assert_eq!(ch.pending(), 0);
    }

    #[test]
    fn clone_handle_shares_queue() {
        let ch1 = channel();
        let ch2 = ch1.clone_handle();
        ch1.send(SentinelMessage::Ping).unwrap();
        assert!(ch2.peek());
        let msg = ch2.recv();
        assert!(msg.is_some());
        assert!(!ch1.peek());
    }
}
