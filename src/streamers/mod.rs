use std::time::Duration;

use crate::Error;
use crate::Result;

mod rx_sync_stream;
pub use rx_sync_stream::*;

mod tx_sync_stream;
pub use tx_sync_stream::*;

pub struct SyncConfig {
    pub(crate) num_buffers: u32,
    pub(crate) buffer_size: u32,
    pub(crate) num_transfers: u32,
    pub(crate) stream_timeout: u32,
}

impl SyncConfig {
    pub fn new(
        num_buffers: u32,
        buffer_size: usize,
        num_transfers: u32,
        stream_timeout: Duration,
    ) -> Result<Self> {
        let stream_timeout = stream_timeout.as_millis().try_into().map_err(|_| {
            Error::msg(format!(
                "Stream timeout to large for u32 millis: {}",
                stream_timeout.as_millis()
            ))
        })?;

        let buffer_size: u32 = buffer_size
            .try_into()
            .map_err(|e| Error::msg(format!("Buffer size too big: {e:?}")))?;

        if buffer_size % 1024 != 0 {
            Err(Error::msg("Buffer size must be a multiple of 1024"))
        } else if num_buffers <= num_transfers {
            Err(Error::msg(
                "Number of buffers must be greater than number of transfers",
            ))
        } else {
            Ok(Self {
                num_buffers,
                buffer_size,
                num_transfers,
                stream_timeout,
            })
        }
    }
}

impl Default for SyncConfig {
    /// Values taken from <https://www.nuand.com/libbladeRF-doc/v2.5.0/sync_no_meta.html>
    fn default() -> Self {
        Self {
            num_buffers: 16,
            buffer_size: 8192,
            num_transfers: 8,
            stream_timeout: 3500,
        }
    }
}
