use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{Extension, AlignmentHashSet};
use super::{Anchors, Anchor};

impl Anchors {
    pub fn get_alignment_results_for_local(self) -> Vec<AlignmentResult> {
        let mut alignment_hash_set = AlignmentHashSet::new();

        self.anchors.into_iter().filter_map(|anchor| {
            anchor.get_optional_alignment_result_of_anchor_for_local(&mut alignment_hash_set)
        }).collect()
    }
}

impl Anchor {
    fn get_optional_alignment_result_of_anchor_for_local(
        self,
        alignment_hash_set: &mut AlignmentHashSet,
    ) -> Option<AlignmentResult> {
        if self.dropped {
            return None;
        }

        let left_extension = self.left_extension.unwrap();
        let right_extension = self.right_extension.unwrap();
        
        let penalty = left_extension.penalty + right_extension.penalty;
        let length = left_extension.length + self.size + right_extension.length;

        let alignment_position_of_record = (
            self.record_position + left_extension.deletion_count as usize - left_extension.length,
            self.record_position + self.size + right_extension.length - right_extension.deletion_count  as usize,
        );       
        let alignment_position_of_query = (
            self.query_position + left_extension.insertion_count as usize - left_extension.length,
            self.query_position + self.size + right_extension.length - right_extension.insertion_count as usize,
        );
        let alignment_position = AlignmentPosition {
            record: alignment_position_of_record,
            query: alignment_position_of_query,
        };

        let alignment_is_new = alignment_hash_set.insert_and_check_new(penalty, alignment_position.clone());
        
        if alignment_is_new {
            let left_operations = left_extension.operations;
            let right_operations = right_extension.operations;

            let alignment_operations = AlignmentOperation::concatenate_operations(left_operations, right_operations, self.size as u32);

            Some(
                AlignmentResult {
                    penalty,
                    length,
                    position: alignment_position,
                    operations: alignment_operations,
                }
            )
        } else {
            None
        }
    }
}