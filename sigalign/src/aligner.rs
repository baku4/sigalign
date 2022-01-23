use crate::{Result, error_msg};
use crate::core::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use crate::reference::{
    Reference, SequenceProvider,
};

// Core algorithms
mod algorithm;
use algorithm::{WaveFront, local_alignment_algorithm, semi_global_alignment_algorithm};

// Common data structures for aligner
//  - Cache for alignment extension
mod wave_front_cache;
use wave_front_cache::{WaveFrontCache, SingleWaveFrontCache, DoubleWaveFrontCache};
//  - Alignment condition
mod alignment_condition;
use alignment_condition::AlignmentCondition;

// Aligner interface
trait AlignerInterface {
    fn new(condition: AlignmentCondition) -> Self where Self: Sized;
    fn alignment<'a, S>(&mut self, reference: &Reference<'a, S>, query: Sequence) -> AlignmentResult where S: SequenceProvider<'a>;
}

// Aligner implementations
mod semi_global;
mod local;
pub use semi_global::SemiGlobalAligner;
pub use local::LocalAligner;

// Features
mod feature;

pub enum Aligner {
    SemiGlobal(SemiGlobalAligner),
    Local(LocalAligner),
}

impl Aligner {
    pub(crate) fn new_semi_global(alignment_condition: AlignmentCondition) -> Self {
        Self::SemiGlobal(SemiGlobalAligner::new(alignment_condition))
    }
    pub(crate) fn new_local(alignment_condition: AlignmentCondition) -> Self {
        Self::Local(LocalAligner::new(alignment_condition))
    }
}

// TODO: to move
// Features for aligner
// mod query_check;
// mod result_translation;
// use query_check::SearchableCheck;

// mod interpreter;
// mod alignment;
// use interpreter::raw_result_to_json;
