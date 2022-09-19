use ahash::AHashMap;
// use std::collections::HashMap;

use super::{
    Aligner,
    Reference,
    ReferenceInterface,
    SequenceBuffer,
    InMemoryProvider,
    PRECISION_SCALE,
    AnchorAlignmentResult,
    PatternLocation,
};

use lt_fm_index::{LtFmIndex, LtFmIndexBuilder};

mod dp_optimal_alignment;
use dp_optimal_alignment::{
    optimal_semi_global_alignment,
    optimal_local_alignment,
};

#[derive(Debug)]
struct DpBasedAligner {
    mismatch_penalty: usize,
    gap_open_penalty: usize,
    gap_extend_penalty: usize,
    minimum_aligned_length: usize,
    maximum_penalty_per_scale: usize,
    pattern_size: usize,
}

impl DpBasedAligner {
    pub fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Self {
        let aligner = Aligner::new_local(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length).unwrap();
        let pattern_size = aligner.get_pattern_size();

        let maximum_penalty_per_scale = (PRECISION_SCALE as f32 * maximum_penalty_per_length) as usize;

        Self {
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            minimum_aligned_length,
            maximum_penalty_per_scale,
            pattern_size,
        }
    }
    pub fn local_alignment(
        &self,
        reference: &Reference<InMemoryProvider>,
        query: &[u8],
    ) {
        //
    }
    fn get_alignment_start_position(
        &self,
        reference: &Reference<InMemoryProvider>,
        query: &[u8],
    ) {
        // Slice query to patterns
        let qry_len = query.len();
        let pattern_size = self.pattern_size;
        let pattern_count = qry_len / pattern_size;

        let mut alignment_start_positions_by_record: AHashMap<usize, Vec<AlignmentStartPosition>> = AHashMap::new();

        for pattern_index in 0..pattern_count {
            let qry_pos = pattern_index * pattern_size;
            let pattern = &query[qry_pos..qry_pos+pattern_size];
            
            let pattern_locations = reference.locate(pattern);

            for pattern_location in pattern_locations {
                let record_index = pattern_location.record_index;
                if let None = alignment_start_positions_by_record.get(&record_index) {
                    alignment_start_positions_by_record.insert(record_index, Vec::new());
                };

                let alignment_start_positions = alignment_start_positions_by_record.get_mut(&record_index).unwrap();

                // FIXME: Write
            }
        }
    }
}

#[derive(Debug)]
struct AlignmentStartPosition {
    record_start_index: usize,
    query_start_index: usize,
    pattern_size: usize,
}
