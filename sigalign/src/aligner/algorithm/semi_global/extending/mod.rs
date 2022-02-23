use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedPosition, TraversedAnchors, TraversedAnchor};
use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};

pub struct ExtensionResult {
    pub extension: Extension,
    pub traversed_anchors: TraversedAnchors,
    pub is_success: bool,
}

impl PosTable {
    pub fn right_extension(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        penalty_margin: i64,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) -> ExtensionResult {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        let record_start_index = anchor_position.record_position + anchor_size;
        let query_start_index = anchor_index.0 * pattern_size + anchor_size;

        let record_slice = &record_sequence[record_start_index..];
        let query_slice = &query_sequence[query_start_index..];

        let spare_penalty = calculate_spare_penalty(penalty_margin, anchor_size, query_slice.len(), record_slice.len(), penalties, cutoff);
        
        wave_front.align_right_to_end_point(record_slice, query_slice, penalties, spare_penalty);

        let (last_penalty, last_component_index, is_success) = match wave_front.end_point.k {
            Some(k) => {
                let end_point_score = wave_front.end_point.score;
                (end_point_score, wave_front.wave_front_scores[end_point_score].index_of_component(k), true)
            },
            None => {
                let (penalty, index) = wave_front.penalty_and_index_of_max_length(penalties.e);
                (penalty, index, false)
            },
        };

        let (extension, traversed_positions) = wave_front.backtrace_from_point_checking_right_traversed(
            last_penalty,
            last_component_index,
            penalties,
            pattern_size,
        );

        let traversed_anchors = self.right_traversed_anchors(
            traversed_positions,
            anchor_index.0,
            anchor_index.1,
            record_start_index,
            extension.length,
            extension.penalty,
            pattern_size,
        );
        
        ExtensionResult {
            extension,
            traversed_anchors,
            is_success,
        }
    }
    pub fn left_extension(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        penalty_margin: i64,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) -> ExtensionResult {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        let record_last_index = anchor_position.record_position;
        let query_last_index = anchor_index.0 * pattern_size;

        let record_slice = &record_sequence[..record_last_index];
        let query_slice = &query_sequence[..query_last_index];

        let spare_penalty = calculate_spare_penalty(penalty_margin, anchor_size, query_slice.len(), record_slice.len(), penalties, cutoff);
        
        wave_front.align_left_to_end_point(record_slice, query_slice, penalties, spare_penalty);

        let (last_penalty, last_component_index, is_success) = match wave_front.end_point.k {
            Some(k) => {
                let end_point_score = wave_front.end_point.score;
                (end_point_score, wave_front.wave_front_scores[end_point_score].index_of_component(k), true)
            },
            None => {
                let (penalty, index) = wave_front.penalty_and_index_of_max_length(penalties.e);
                (penalty, index, false)
            },
        };

        let (extension, traversed_positions) = wave_front.backtrace_from_point_checking_left_traversed(
            last_penalty,
            last_component_index,
            penalties,
            pattern_size,
        );

        let traversed_anchors = self.left_traversed_anchors(
            traversed_positions,
            anchor_index.0,
            record_last_index,
            extension.length,
            extension.penalty,
            pattern_size,
        );
        
        ExtensionResult {
            extension,
            traversed_anchors,
            is_success,
        }
    }
}

impl WaveFront {
    fn penalty_and_index_of_max_length(&self, gap_extend_penalty: usize) -> (usize, usize) {
        let mut end_point_score = self.end_point.score;
        let mut end_point_index = 0;

        let mut max_length = i32::MIN;
        for i in 0..gap_extend_penalty {
            let score = self.end_point.score - i;
            let wave_front_score = &self.wave_front_scores[score];
            let (index, max_length_of_score) = wave_front_score.index_and_maximum_length();
            if max_length > max_length_of_score {
                max_length = max_length_of_score;
                end_point_score = score;
                end_point_index = index;
            }
        };
        
        (end_point_score, end_point_index)
        // match self.end_point.k {
        //     Some(k) => {
        //         let end_point_score = self.end_point.score;
        //         (end_point_score, self.wave_front_scores[end_point_score].index_of_component(k));
        //     },
        // }
    }
}

impl WaveFrontScore {
    fn index_and_maximum_length(&self) -> (usize, i32) {
        let optional_index_and_maximum_length = self.components_by_k.iter().enumerate()
            .filter_map(|(component_index, components)| {
                match components.m.optional_length() {
                    Some(length) => Some((component_index, length)),
                    None => None,
                }
            }).max_by_key(|(_, length)| *length);

        match optional_index_and_maximum_length {
            Some(v) => {
                v
            },
            None => {
                (0, 0)
            },
        }
    }
}

impl Component {
    fn optional_length(&self) -> Option<i32> {
        match self.bt {
            BackTraceMarker::Empty => None,
            _ => Some(self.fr + self.deletion_count as i32)
        }
    }
}
