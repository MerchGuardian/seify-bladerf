//! Structures related to expansion boards.
//!
//! [Nuand's Product Page](https://www.nuand.com/product-category/expansion-boards/)
//!
//! Some relavent pages from the `libbladerf` docs:
//! - <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___x_b.html>
//! - <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___b_l_a_d_e_r_f1___x_b.html>
//! - <https://www.nuand.com/libbladeRF-doc/v2.5.0/group___f_n___e_x_p___i_o.html>

mod xb200;
pub use xb200::*;

mod xb200_filter;
pub use xb200_filter::*;

mod xb200_path;
pub use xb200_path::*;

pub(crate) mod xb_gpio;
pub mod xb_gpio_impls;
