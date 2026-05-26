// Copyright (c) 2026 workingbb9-official
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

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
