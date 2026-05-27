// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 workingbb9-official

use crate::device::Device;

#[derive(Debug)]
pub(crate) struct Peer<Addr> {
    pub dev: Device,
    pub addr: Addr,
    last_seen_ms: u32,
    last_sent_ms: u32,
    timeout_ms: u32,
}

impl<Addr> Peer<Addr> {
    pub fn new(dev: Device, addr: Addr, now: u32, timeout_ms: u32) -> Self {
        Self {
            dev,
            addr,
            last_seen_ms: now,
            last_sent_ms: now,
            timeout_ms,
        }
    }

    #[inline]
    pub fn is_timed_out(&self, now: u32) -> bool {
        now.wrapping_sub(self.last_seen_ms) >= self.timeout_ms
    }

    #[inline]
    pub fn needs_heartbeat(&self, now: u32, send_interval: u32) -> bool {
        now.wrapping_sub(self.last_sent_ms) >= send_interval
    }

    #[inline]
    pub fn update_last_seen(&mut self, now: u32) {
        self.last_seen_ms = now;
    }

    #[inline]
    pub fn update_last_sent(&mut self, now: u32) {
        self.last_sent_ms = now;
    }
}
