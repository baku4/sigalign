use super::{Result, error_msg};

#[cfg(not(any(
    feature = "json",
    feature = "thread",
    feature = "non_rc",
)))]
mod tsv_alignment_app;
#[cfg(not(any(
    feature = "json",
    feature = "thread",
    feature = "non_rc",
)))]
pub use tsv_alignment_app::AlignmentApp;

#[cfg(feature = "json")]
mod json_alignment_app;
#[cfg(feature = "json")]
pub use json_alignment_app::AlignmentApp;

#[cfg(feature = "thread")]
mod pool_alignment_app;
#[cfg(feature = "thread")]
pub use pool_alignment_app::AlignmentApp;

#[cfg(feature = "non_rc")]
mod non_rc_alignment_app;
#[cfg(feature = "non_rc")]
pub use non_rc_alignment_app::AlignmentApp;
