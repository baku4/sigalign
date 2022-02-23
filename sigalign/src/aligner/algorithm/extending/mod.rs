use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

// Wavefront structure for alignment
mod wave_front;
pub use wave_front::{WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

// Extension
#[derive(Debug, Clone)]
pub struct Extension {
    pub penalty: usize,
    pub length: usize,
    pub insertion_count: u32,
    pub deletion_count: u32,
    pub operations: Vec<AlignmentOperation>,
}

pub fn calculate_spare_penalty_from_determinant(
    spare_penalty_determinant_of_other_side: i64,
    anchor_size: usize,
    query_length_this_side: usize,
    record_length_this_side: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
) -> usize {
    i64::max(
        penalties.o as i64,
        (
            penalties.e as i64 * spare_penalty_determinant_of_other_side
            + cutoff.maximum_penalty_per_scale as i64 * (
                (
                    penalties.e * (
                        anchor_size + query_length_this_side.min(record_length_this_side)
                    )
                ) as i64 - penalties.o as i64
            )
        ) / (
            PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
        ) as i64 + 1
    ) as usize
}

impl AlignmentOperation {
    pub fn concatenate_operations(
        mut left_operations: Vec<Self>,
        mut right_operations: Vec<Self>,
        anchor_size: u32
    ) -> Vec<Self> {
        right_operations.reverse();

        // Add anchor sized Match operation to left operations
        if let Some(
            AlignmentOperation {
                case: AlignmentCase::Match,
                count,
            }
        ) = left_operations.last_mut() {
            *count += anchor_size;
        } else {
            left_operations.push(
                AlignmentOperation {
                    case: AlignmentCase::Match,
                    count: anchor_size,
                }
            );
        };

        // Add right operations to left operations
        if let Some(
            AlignmentOperation {
                case: AlignmentCase::Match,
                count: right_count,
            }
        ) = right_operations.first_mut() {
            if let AlignmentOperation {
                case: AlignmentCase::Match,
                count: left_count,
            } = left_operations.last_mut().unwrap() {
                *left_count += *right_count;
            }
            right_operations.remove(0);
        };

        left_operations.append(&mut right_operations);

        left_operations
    }
}
