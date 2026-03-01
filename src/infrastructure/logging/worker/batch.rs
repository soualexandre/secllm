//! Dynamic batch: flush when max_size reached or max_latency elapsed.

use crate::domain::AuditEvent;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct DynamicBatch {
    buffer: VecDeque<AuditEvent>,
    max_size: usize,
    max_latency: Duration,
    first_at: Option<Instant>,
}

impl DynamicBatch {
    pub fn new(max_size: usize, max_latency_ms: u64) -> Self {
        Self {
            buffer: VecDeque::new(),
            max_size,
            max_latency: Duration::from_millis(max_latency_ms),
            first_at: None,
        }
    }

    pub fn push(&mut self, event: AuditEvent) {
        if self.first_at.is_none() {
            self.first_at = Some(Instant::now());
        }
        self.buffer.push_back(event);
    }

    /// Returns a batch if ready: size >= max_size or latency >= max_latency.
    pub fn take_ready(&mut self) -> Option<Vec<AuditEvent>> {
        let ready = self.ready();
        if !ready {
            return None;
        }
        self.first_at = None;
        let out: Vec<AuditEvent> = self.buffer.drain(..).collect();
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn ready(&self) -> bool {
        if self.buffer.is_empty() {
            return false;
        }
        if self.buffer.len() >= self.max_size {
            return true;
        }
        self.first_at
            .map(|t| t.elapsed() >= self.max_latency)
            .unwrap_or(false)
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}
