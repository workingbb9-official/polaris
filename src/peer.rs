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

pub(crate) enum PeerState {
    Authorized,
    Unauthorized,
}

pub(crate) struct Peer<Addr> {
    pub(crate) dev: Device,
    pub(crate) addr: Addr,
    last_seen_ms: u32,
    last_sent_ms: u32,
    timeout_ms: u32,
    send_interval_ms: u32,
}

impl<Addr> Peer<Addr> {
    pub(crate) fn new(dev: Device, addr: Addr, now: u32, timeout_ms: u32, send_interval_ms: u32) -> Self {
        Self {
            dev,
            addr,
            last_seen_ms: now,
            last_sent_ms: now,
            timeout_ms,
            send_interval_ms,
        }
    }

    #[inline]
    pub(crate) fn is_timed_out(&self, now: u32) -> bool {
        now.wrapping_sub(self.last_seen_ms) >= self.timeout_ms
    }

    #[inline]
    pub(crate) fn needs_heartbeat(&self, now: u32) -> bool {
        now.wrapping_sub(self.last_sent_ms) >= self.send_interval_ms
    }
}
