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

mod interpreter;
mod alignment;
use interpreter::raw_result_to_json;
// TODO: to move

mod alignment_condition;
use alignment_condition::AlignmentCondition;
mod wave_front_cache;

mod semi_global;
mod local;



fn calculate_max_score_from_length(
    penalties: &Penalties,
    cutoff: &Cutoff,
    query_length: usize,
) -> usize {
    (
        cutoff.maximum_penalty_per_scale * (
            penalties.e * query_length - penalties.o
        )
    ) / (
        PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
    ) + 1
}

// impl WaveFrontHolder {
//     const QUERY_LENGTH_UNIT: usize = 100;

//     fn new(
//         penalties: &Penalties,
//         cutoff: &Cutoff,
//     ) -> Self {
//         let to_allocate_query_length = Self::QUERY_LENGTH_UNIT;
//         let max_score = Self::calculate_max_score_from_length(penalties, cutoff, to_allocate_query_length);

//         let allocated_wave_front = WaveFront::new_allocated(penalties, max_score);

//         Self {
//             allocated_query_length: to_allocate_query_length,
//             primary_wave_front: allocated_wave_front.clone(),
//             secondary_wave_front: allocated_wave_front,
//         }
//     }
//     fn calculate_max_score_from_length(
//         penalties: &Penalties,
//         cutoff: &Cutoff,
//         query_length: usize,
//     ) -> usize {
//         (
//             cutoff.maximum_penalty_per_scale * (
//                 penalties.e * query_length - penalties.o
//             )
//         ) / (
//             PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
//         ) + 1
//     }
// }

// use std::fmt;
// impl fmt::Debug for WaveFrontHolder {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("WaveFrontHolder")
//          .field("allocated_query_length", &self.allocated_query_length)
//          .finish()
//     }
// }
