// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 workingbb9-official

use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub(crate) const DEVICE_ID_LEN: usize = 2;

/// Metadata for all devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Device {
    id: DeviceId,
    kind: DeviceType,
}

impl Device {
    pub fn new(id: DeviceId, kind: DeviceType) -> Self {
        Self { id, kind }
    }

    /// Access the [DeviceId].
    #[inline]
    pub fn id(&self) -> DeviceId {
        self.id
    }

    /// Access the [DeviceType].
    #[inline]
    pub fn kind(&self) -> DeviceType {
        self.kind
    }
}

/// A unique identifier for every [Device].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct DeviceId(u16);

impl DeviceId {
    #[inline]
    pub fn new(val: u16) -> Self {
        Self(val)
    }

    /// Accesses the numeric value of the ID.
    #[inline]
    pub fn value(&self) -> u16 {
        self.0
    }
}

/// Describes what a [Device] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, FromBytes, IntoBytes, Immutable, KnownLayout)]
#[repr(transparent)]
pub struct DeviceType(u16);

impl DeviceType {
    #[inline]
    pub fn new(val: u16) -> Self {
        Self(val)
    }

    /// Accesses the numeric value of the type.
    #[inline]
    pub fn value(&self) -> u16 {
        self.0
    }
}
