use crate::{Result, error_msg};
use crate::core::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use crate::utils::FastaReader;
use crate::algorithm::{WaveFront, local_alignment_algorithm, semi_global_alignment_algorithm};
use crate::reference::{Reference, SequenceProvider, Labeling};

// Common data structures for aligner
mod alignment_condition;
mod wave_front_cache;
use alignment_condition::AlignmentCondition;
use wave_front_cache::{WaveFrontCache, SingleWaveFrontCache, DoubleWaveFrontCache};

// Aligner implementations
mod semi_global;
mod local;
pub use semi_global::SemiGlobalAligner;
pub use local::LocalAligner;

// TODO: to move
// Features for aligner
// mod query_check;
// mod result_translation;
// use query_check::SearchableCheck;

// mod interpreter;
// mod alignment;
// use interpreter::raw_result_to_json;
