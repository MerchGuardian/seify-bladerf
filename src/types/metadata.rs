use crate::sys::*;

/// Additional types for Metadata
#[derive(Clone, Debug)]
pub struct Metadata {
    pub timestamp: u64,
    pub flags: u32,
    // Add other fields as necessary
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            timestamp: 0,
            flags: 0,
        }
    }
}

impl From<&bladerf_metadata> for Metadata {
    fn from(meta: &bladerf_metadata) -> Self {
        Self {
            timestamp: meta.timestamp,
            flags: meta.flags,
        }
    }
}

impl From<&Metadata> for bladerf_metadata {
    fn from(val: &Metadata) -> Self {
        bladerf_metadata {
            timestamp: val.timestamp,
            flags: val.flags,
            status: 0,
            actual_count: 0,
            reserved: [0u8; 32],
        }
    }
}
