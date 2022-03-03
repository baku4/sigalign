use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

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
