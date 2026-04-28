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

pub(crate) struct DiscoveryManager {
    id: DeviceId,
}

impl DiscoveryManager {
    pub(crate) fn new(id: DeviceId) -> Self {
        Self { id }
    }

    pub(crate) fn process<T: Transport>(
        &self,
        transport: &mut T,
    ) -> Result<(), DiscoveryError<T::Error>> {
        self.send(transport)
    }

    fn send<T: Transport>(&self, transport: &mut T) -> Result<(), DiscoveryError<T::Error>> {
        let msg = DiscoveryMessage::new_hello(self.id);
        let raw = msg.to_bytes();

        let addr = transport.broadcast_addr();

        match transport.send(&raw, &addr) {
            Ok(()) => Ok(()),
            Err(e) => Err(DiscoveryError::Transport(e)),
        }
    }
}
