use crate::{Result, error_msg};
use crate::core::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
pub use crate::reference::{
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
pub use alignment_condition::AlignmentCondition;

// Aligner interface
pub trait AlignerInterface {
    fn new(condition: AlignmentCondition) -> Self where Self: Sized;
    fn alignment<S>(
        &mut self,
        reference: &Reference<S>,
        sequence_buffer: &mut S::Buffer,
        query: Sequence,
    ) -> AlignmentResult where S: SequenceProvider;
}

// Aligner implementations
mod semi_global;
mod local;
pub use semi_global::SemiGlobalAligner;
pub use local::LocalAligner;

// Features
mod feature;

#[derive(Clone)]
pub struct Aligner {
    pub(crate) algorithms: Algorithms,
}

#[derive(Clone)]
pub enum Algorithms {
    SemiGlobal(SemiGlobalAligner),
    Local(LocalAligner),
}

impl Aligner {
    pub fn new_semi_global(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Self> {
        let alignment_condition = AlignmentCondition::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        Ok(Self {
            algorithms: Algorithms::SemiGlobal(SemiGlobalAligner::new(alignment_condition))
        })
    }
    pub fn new_local(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Self> {
        let alignment_condition = AlignmentCondition::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        Ok(Self {
            algorithms: Algorithms::Local(LocalAligner::new(alignment_condition))
        })
    }
    pub fn alignment<S: SequenceProvider>(
        &mut self,
        reference: &Reference<S>,
        sequence_buffer: &mut S::Buffer,
        query: Sequence,
    ) -> AlignmentResult {
        match &mut self.algorithms {
            Algorithms::SemiGlobal(aligner) => aligner.alignment(reference, sequence_buffer, query),
            Algorithms::Local(aligner) => aligner.alignment(reference, sequence_buffer, query),
        }
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
