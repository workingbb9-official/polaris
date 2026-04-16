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

use crate::device::Device;
use crate::{Addr, Transport};

/// A device which reports to a `Controller`.
///
/// Nodes will remain simple and able to describe any type of device. They can be anything that
/// operates independently, and sends information to a Controller.
pub struct Node<T: Transport> {
    dev: Device,
    controller: Option<Addr>,
    transport: T,
}

impl<T: Transport> Node<T> {
    /// Creates a new Node.
    pub fn new(dev: Device, transport: T) -> Self {
        Self {
            dev,
            controller: None,
            transport,
        }
    }

    /// Extract the [Device] of the node.
    #[inline]
    pub fn dev(&self) -> Device {
        self.dev
    }
}
