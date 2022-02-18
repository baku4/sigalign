use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use std::collections::HashMap;

// Sorted record positions by pattern
#[derive(Debug, Clone)]
pub struct PosTable{
    pub position_by_pattern: Vec<Option<PatternPosition>>,
}

#[derive(Debug, Clone)]
pub struct PatternPosition {
    pub anchor_positions: Vec<AnchorPosition>,
}

#[derive(Debug, Clone)]
pub struct AnchorPosition {
    pub record_position: usize,
    pub pattern_count: usize,
}

impl PosTable {
    pub fn new_by_record<S: SequenceProvider>(
        reference: &Reference<S>,
        query: Sequence,
        pattern_size: usize,
    ) -> Vec<(usize, Self)> {
        let qry_len = query.len();
        let pattern_count = qry_len / pattern_size;

        let mut pos_table_map_by_record: HashMap<usize, Self> = HashMap::new();

        for pattern_index in 0..pattern_count {
            let qry_pos = pattern_index * pattern_size;
            let pattern = &query[qry_pos..qry_pos+pattern_size];

            let reference_location = reference.locate(pattern);

            for record_location in reference_location {
                match pos_table_map_by_record.get_mut(&record_location.record_index) {
                    Some(pos_table) => {
                        pos_table.add_new_positions(pattern_index, record_location.positions)
                    },
                    None => {
                        let mut new_pos_table = Self::new_empty(pattern_count);
                        new_pos_table.add_new_positions(pattern_index, record_location.positions);
                        pos_table_map_by_record.insert(record_location.record_index, new_pos_table);
                    }
                }
            }
        }

        pos_table_map_by_record.into_iter().map(|(record_index, pos_table)| {
            (record_index, pos_table.merge_ungapped_anchors(pattern_size))
        }).collect()
    }
    // For New
    fn new_empty(pattern_count: usize) -> Self {
        Self {
            position_by_pattern: vec![None; pattern_count],
        }
    }
    fn add_new_positions(
        &mut self,
        pattern_index: usize,
        sorted_record_positions: Vec<usize>,
    ) {
        let pattern_position = PatternPosition::new(sorted_record_positions);
        self.position_by_pattern[pattern_index] = Some(pattern_position);
    }
    // For Merge
    fn merge_ungapped_anchors(mut self, pattern_size: usize) -> Self {
        let pattern_count = self.position_by_pattern.len();

        let mut merged = Self::new_empty(pattern_count);

        let mut left_pattern_position = None;
        let mut right_pattern_position = self.position_by_pattern[pattern_count-1].take();

        for left_index in 0..pattern_count-1 {
            left_pattern_position = self.position_by_pattern[left_index].take();

            PatternPosition::merge_right_to_left(
                &mut left_pattern_position,
                &mut right_pattern_position,
                pattern_size,
            );

            merged.position_by_pattern[left_index + 1] = right_pattern_position.take();
            right_pattern_position = left_pattern_position.take();
        }

        merged.position_by_pattern[0] = right_pattern_position.take();

        merged
    }
}

impl PatternPosition {
    fn new(sorted_record_positions: Vec<usize>) -> Self {
        Self {
            anchor_positions: AnchorPosition::new_of_vector(sorted_record_positions),
        }
    }

    fn merge_right_to_left(left: &mut Option<Self>, right: &mut Option<Self>, pattern_size: usize) {
        if let (Some(left), Some(right)) = (left, right) {
            // TODO: Next
        }
    }
}

impl AnchorPosition {
    fn new_of_vector(sorted_record_positions: Vec<usize>) -> Vec<Self> {
        sorted_record_positions.into_iter().map(|pos| {
            Self {
                record_position: pos,
                pattern_count: 1,
            }
        }).collect()
    }
}
