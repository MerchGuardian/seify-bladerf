use std::time::Duration;

use crate::Error;
use crate::Result;

mod rx_sync_stream;
pub use rx_sync_stream::*;

mod tx_sync_stream;
pub use tx_sync_stream::*;

/// Configuration parameters for a stream of samples.
///
/// # Related Links on Nuand's Site
/// - <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_t_r_e_a_m_i_n_g___s_y_n_c.html#ga6f76857ec83bc56d485842dd55eebe65>
/// - <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_t_r_e_a_m_i_n_g___a_s_y_n_c.html#ga72752f2a047b95544e7686596a409abd>
#[derive(Debug, Clone, Copy)]
pub struct StreamConfig {
    pub(crate) num_buffers: u32,
    pub(crate) buffer_size: u32,
    pub(crate) num_transfers: u32,
    pub(crate) stream_timeout: u32,
}

impl StreamConfig {
    /// Creates a new [StreamConfig] that can be used to configure streams like [RxSyncStream] and [TxSyncStream]
    ///
    /// # Errors
    /// - The `buffer_size` must be a multiple of 1024.
    /// - `num_buffers` must be >= `num_transfers`
    /// - When the parameters are not able to be properly converted into a [u32] since that is what is used by `libbladerf` internally.
    ///
    /// It is reccomended to look at Nuand's docs for [`bladerf_init_stream()`](https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___s_t_r_e_a_m_i_n_g___a_s_y_n_c.html#ga72752f2a047b95544e7686596a409abd)
    /// to get some more information on setting those parameters.
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

impl Default for StreamConfig {
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
