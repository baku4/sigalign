use super::Result;

#[cfg(not(any(
    feature = "segment",
    feature = "thread",
)))]
mod tsv_alignment_app;
#[cfg(not(any(
    feature = "segment",
    feature = "thread",
)))]
pub use tsv_alignment_app::AlignmentApp;

#[cfg(feature = "segment")]
mod segment_alignment_app;
#[cfg(feature = "segment")]
pub use segment_alignment_app::AlignmentApp;

#[cfg(feature = "thread")]
mod pool_alignment_app;
#[cfg(feature = "thread")]
pub use pool_alignment_app::AlignmentApp;
