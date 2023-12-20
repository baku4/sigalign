use sigalign_core::aligner::AlignmentRegulator;

mod dynamic_aligner;
use dynamic_aligner::DynamicAligner;

mod build;
pub use build::AlignerBuildError;
mod perform_alignments;
mod switch_algorithm;
mod debug;

/// An alignment executor.
#[derive(Clone)]
pub struct Aligner {
    regulator: AlignmentRegulator,
    dynamic_aligner: DynamicAligner,
}
