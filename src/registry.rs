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

use crate::peer::Peer;

pub struct Registry<Addr, const MAX_PEERS: usize> {
    peers: heapless::Vec<PeerSession<Addr>, MAX_PEERS>,
}

impl<Addr, const MAX_PEERS: usize> Registry<Addr, MAX_PEERS> {
    pub(crate) fn new() -> Self {
        Self { peers: heapless::Vec::new() }
    }

    pub(crate) fn add_peer(&mut self, peer: Peer) Result<(), ()> {
        let id = peer.dev.id();
        let addr = peer.addr;

        if list.iter().any(|peer| peer.dev.id() == id) {
            return Err(())
        }

        self.peers.push().map_err()
    }
}
