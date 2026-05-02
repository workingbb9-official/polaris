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

#![allow(dead_code)]

use crate::device::DeviceId;
use crate::protocol::HeartbeatMessage;
use crate::transport::Transport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeartbeatError<TE> {
    Transport(TE),
}

#[derive(Debug)]
pub(crate) struct HeartbeatManager {
    node_id: DeviceId,
    send_interval: u32,
    last_sent: u32,
}

impl HeartbeatManager {
    pub(crate) fn new(node_id: DeviceId, send_interval: u32) -> Self {
        Self {
            node_id,
            send_interval,
            last_sent: 0,
        }
    }

    pub(crate) fn process<T: Transport>(
        &mut self,
        transport: &mut T,
        controller_addr: &T::Addr,
        now: u32,
    ) -> Result<(), HeartbeatError<T::Error>> {
        if now.wrapping_sub(self.last_sent) >= self.send_interval {
            self.last_sent = now;
            self.send_heartbeat(transport, controller_addr)
        } else {
            Ok(())
        }
    }

    fn send_heartbeat<T: Transport>(
        &self,
        transport: &mut T,
        controller_addr: &T::Addr,
    ) -> Result<(), HeartbeatError<T::Error>> {
        let msg = HeartbeatMessage::new(self.node_id);
        let raw = msg.to_bytes();

        match transport.send(&raw, controller_addr) {
            Ok(()) => Ok(()),
            Err(e) => Err(HeartbeatError::Transport(e)),
        }
    }
}
