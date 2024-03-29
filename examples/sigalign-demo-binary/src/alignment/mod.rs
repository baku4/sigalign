// Default alignment app
#[cfg(not(any(
    feature = "thread",
    feature = "chain",
)))]
mod default_alignment_app;
#[cfg(not(any(
    feature = "thread",
    feature = "chain",
)))]
pub use default_alignment_app::AlignmentApp;

// Alignment with thread
#[cfg(feature = "thread")]
mod pool_alignment_app;
#[cfg(feature = "thread")]
pub use pool_alignment_app::AlignmentApp;

// Alignment with multiple cutoffs
#[cfg(feature = "chain")]
mod chaining_alignment_app;
#[cfg(feature = "chain")]
pub use chaining_alignment_app::AlignmentApp;

mod write_results;
pub use write_results::{
    ForwardDirection, ReverseDirection,
    write_alignment_result_as_tsv,
};
