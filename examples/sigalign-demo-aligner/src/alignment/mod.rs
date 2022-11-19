use super::{Result, error_msg};

#[cfg(not(any(
    feature = "tsv",
    feature = "thread",
    feature = "rc_qry",
)))]
mod default_alignment;
#[cfg(not(any(
    feature = "tsv",
    feature = "thread",
    feature = "rc_qry",
)))]
pub use default_alignment::AlignmentApp;


#[cfg(feature = "tsv")]
mod tsv_alignment;
#[cfg(feature = "tsv")]
pub use tsv_alignment::AlignmentApp;

#[cfg(feature = "thread")]
mod pool_alignment;
#[cfg(feature = "thread")]
pub use pool_alignment::AlignmentApp;

#[cfg(feature = "rc_qry")]
mod rc_qry_alignment;
#[cfg(feature = "rc_qry")]
pub use rc_qry_alignment::AlignmentApp;