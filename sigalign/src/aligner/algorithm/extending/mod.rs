use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, calculate_spare_penalty};

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

impl PosTable {
    pub fn extend_wave_front_right(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        penalty_margin: i64,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        let record_start_index = anchor_position.record_position + anchor_size;
        let query_start_index = anchor_index.0 * pattern_size + anchor_size;

        let record_slice = &record_sequence[record_start_index..];
        let query_slice = &query_sequence[query_start_index..];

        let spare_penalty = calculate_spare_penalty(penalty_margin, anchor_size, query_slice.len(), record_slice.len(), penalties, cutoff);
        
        wave_front.align_right_to_end_point(record_slice, query_slice, penalties, spare_penalty);
    }
    pub fn extend_wave_front_left(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        penalty_margin: i64,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        let record_last_index = anchor_position.record_position;
        let query_last_index = anchor_index.0 * pattern_size;

        let record_slice = &record_sequence[..record_last_index];
        let query_slice = &query_sequence[..query_last_index];

        let spare_penalty = calculate_spare_penalty(penalty_margin, anchor_size, query_slice.len(), record_slice.len(), penalties, cutoff);
        
        wave_front.align_left_to_end_point(record_slice, query_slice, penalties, spare_penalty);
    }
}