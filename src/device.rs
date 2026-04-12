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

/// Metada for all devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Device {
    id: DeviceId,
    device_type: DeviceType,
}

impl Device {
    pub(crate) fn new(id: DeviceId, device_type: DeviceType) -> Self {
        Self { id, device_type }
    }

    pub(crate) fn id(&self) -> DeviceId {
        self.id
    }

    pub(crate) fn device_type(&self) -> DeviceType {
        self.device_type
    }
}

/// A unique identifier for every [Device].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceId(u64);

impl DeviceId {
    /// Creates a new DeviceId.
    #[inline]
    pub fn new(val: u64) -> Self {
        Self(val)
    }

    /// Accesses the numeric value of the ID.
    #[inline]
    pub fn value(&self) -> u64 {
        self.0
    }
}

/// Describes what a [Device] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceType(u64);
