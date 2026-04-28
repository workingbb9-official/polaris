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
use crate::protocol::DiscoveryMessage;
use crate::transport::Transport;

pub enum DiscoveryError<TE> {
    Transport(TE),
}

pub(crate) enum DiscoveryStatus<A> {
    Broadcasted,
    Listening,
    Found(A),
}

pub(crate) struct DiscoveryManager {
    id: DeviceId,
    send_interval: u32,
    last_sent: u32,
}

impl DiscoveryManager {
    pub(crate) fn new(id: DeviceId, send_interval: u32) -> Self {
        Self {
            id,
            send_interval,
            last_sent: 0,
        }
    }

    pub(crate) fn process<T: Transport>(
        &mut self,
        transport: &mut T,
        now: u32,
    ) -> Result<DiscoveryStatus<T::Addr>, DiscoveryError<T::Error>> {
        if now.wrapping_sub(self.last_sent) >= self.send_interval {
            self.last_sent = now;
            match self.broadcast(transport) {
                Ok(()) => Ok(DiscoveryStatus::Broadcasted),
                Err(e) => Err(e),
            }
        } else {
            match self.listen(transport) {
                Ok(Some(addr)) => Ok(DiscoveryStatus::Found(addr)),
                Ok(None) => Ok(DiscoveryStatus::Listening),
                Err(e) => Err(e),
            }
        }
    }

    fn broadcast<T: Transport>(&self, transport: &mut T) -> Result<(), DiscoveryError<T::Error>> {
        let msg = DiscoveryMessage::new_hello(self.id);
        let raw = msg.to_bytes();

        let addr = transport.broadcast_addr();

        match transport.send(&raw, &addr) {
            Ok(()) => Ok(()),
            Err(e) => Err(DiscoveryError::Transport(e)),
        }
    }

    fn listen<T: Transport>(
        &self,
        transport: &mut T,
    ) -> Result<Option<T::Addr>, DiscoveryError<T::Error>> {
        let mut buf = [0u8; 8];

        let addr = match transport.recv(&mut buf) {
            Ok((_, addr)) => addr,
            Err(e) => return Err(DiscoveryError::Transport(e)),
        };

        if DiscoveryMessage::from_bytes(&buf).is_none() {
            Ok(None)
        } else {
            Ok(Some(addr))
        }
    }
}
