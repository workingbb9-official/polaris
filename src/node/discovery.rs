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

#[derive(Debug, PartialEq)]
pub(crate) enum DiscoveryAction {
    Broadcast,
    None,
}

#[derive(Debug)]
pub(crate) struct DiscoveryManager {
    send_interval: u32,
    last_sent: u32,
}

impl DiscoveryManager {
    pub(crate) fn new(send_interval: u32) -> Self {
        Self {
            send_interval,
            last_sent: 0,
        }
    }

    pub(crate) fn action(&mut self, now: u32) -> DiscoveryAction {
        if now.wrapping_sub(self.last_sent) >= self.send_interval {
            DiscoveryAction::Broadcast
        } else {
            DiscoveryAction::None
        }
    }
}
