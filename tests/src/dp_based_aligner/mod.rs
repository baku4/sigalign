use ahash::{AHashMap, AHashSet};

use super::{
    Aligner,
    Reference,
    ReferenceInterface,
    SequenceBuffer,
    InMemoryProvider,
    PRECISION_SCALE,
    AlignmentResult,
    RecordAlignmentResult,
    AnchorAlignmentResult,
    AlignmentOperation,
    AlignmentCase,
    PatternLocation,
};

mod dp_optimal_alignment;
use dp_optimal_alignment::{
    optimal_semi_global_alignment,
    optimal_local_alignment,
};

#[derive(Debug)]
pub struct DpBasedAligner {
    pub mismatch_penalty: usize,
    pub gap_open_penalty: usize,
    pub gap_extend_penalty: usize,
    pub minimum_aligned_length: usize,
    pub maximum_penalty_per_scale: usize,
    pub pattern_size: usize,
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
    pub fn semi_global_alignment(
        &self,
        reference: &Reference<InMemoryProvider>,
        query: &[u8],
    ) -> AlignmentResult {
        self.do_alignment(
            reference,
            query,
            optimal_semi_global_alignment,
        )
    }
    pub fn local_alignment(
        &self,
        reference: &Reference<InMemoryProvider>,
        query: &[u8],
    ) -> AlignmentResult {
        self.do_alignment(
            reference,
            query,
            optimal_local_alignment,
        )
    }
    fn do_alignment<F>(
        &self,
        reference: &Reference<InMemoryProvider>,
        query: &[u8],
        alignment_algorithm: F,
    ) -> AlignmentResult where
        F: Fn(&[u8], &[u8], usize, usize, usize, usize, usize, usize, usize, usize) ->  Option<AnchorAlignmentResult>
    {
        let alignment_start_positions_by_record = self.get_alignment_start_position(reference, query);
        let mut sequence_buffer = reference.get_buffer();

        let mut record_alignment_results = Vec::new();

        for (record_index, alignment_start_positions) in alignment_start_positions_by_record {
            reference.fill_sequence_buffer(record_index, &mut sequence_buffer);
            let record = sequence_buffer.request_sequence();

            let anchor_alignment_results: Vec<AnchorAlignmentResult> = alignment_start_positions.into_iter().filter_map(|alignment_start_position| {
                alignment_algorithm(
                    record,
                    query,
                    alignment_start_position.record_start_position, // record_start_position
                    alignment_start_position.query_start_position, // query_start_position
                    self.pattern_size,
                    self.mismatch_penalty,
                    self.gap_open_penalty,
                    self.gap_extend_penalty,
                    self.minimum_aligned_length,
                    self.maximum_penalty_per_scale,
                )
            }).collect();

            if anchor_alignment_results.len() != 0 {
                let unique_alignments = get_unique_alignments(anchor_alignment_results);

                let record_alignment_result = RecordAlignmentResult {
                    index: record_index,
                    alignments: unique_alignments,
                };

                record_alignment_results.push(record_alignment_result);
            }
        }
        AlignmentResult(record_alignment_results)
    }
    fn get_alignment_start_position(
        &self,
        reference: &Reference<InMemoryProvider>,
        query: &[u8],
    ) -> AHashMap<usize, Vec<AlignmentStartPosition>> {
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

                for record_position in pattern_location.positions{
                    let new_alignment_start_position = AlignmentStartPosition {
                        record_start_position: record_position,
                        query_start_position: qry_pos,
                        pattern_size: pattern_size,
                    };
                    AlignmentStartPosition::add_new_position(alignment_start_positions, new_alignment_start_position);
                }
            }
        }

        alignment_start_positions_by_record
    }
}

#[derive(Debug)]
struct AlignmentStartPosition {
    record_start_position: usize,
    query_start_position: usize,
    pattern_size: usize,
}
impl AlignmentStartPosition {
    fn add_new_position(vec: &mut Vec<Self>, new: Self) {
        let mut extended = false;
        for v in vec.iter_mut() {
            if (
                v.record_start_position + v.pattern_size == new.record_start_position
            ) && (
                v.query_start_position + v.pattern_size == new.query_start_position
            ) {
                v.pattern_size += new.pattern_size;
                extended = true;
                break;
            }
        }
        if !extended {
            vec.push(new);
        }
    }
}

fn get_unique_alignments(mut anchor_alignment_results: Vec<AnchorAlignmentResult>) -> Vec<AnchorAlignmentResult> {
    // Sort
    anchor_alignment_results.sort_by(|a,b| {
        let (a_ql, a_p) = get_query_length_and_penalty(a);
        let (b_ql, b_p) = get_query_length_and_penalty(b);
        if a_ql == b_ql {
            a_p.partial_cmp(&b_p).unwrap()
        } else {
            b_ql.partial_cmp(&a_ql).unwrap()
        }
    });

    // Sort out
    let mut registered_position: AHashSet<(usize, usize)> = AHashSet::new(); // record index, query index
    let mut unique_alignments = Vec::new();
    for anchor_alignment_result in anchor_alignment_results {
        let base_pair_position_set = get_base_pair_position_set(&anchor_alignment_result);
        if registered_position.is_disjoint(&base_pair_position_set) {
            let new_registered_position = registered_position.union(&base_pair_position_set).map(|v| *v).collect();
            registered_position = new_registered_position;

            unique_alignments.push(anchor_alignment_result);
        }
    }

    unique_alignments
}

fn get_query_length_and_penalty(anchor_alignment_result: &AnchorAlignmentResult) -> (usize, usize) {
    let query_length = anchor_alignment_result.operations.iter().map(|op| {
        match op.case {
            AlignmentCase::Match
            | AlignmentCase::Subst
            | AlignmentCase::Deletion => op.count as usize,
            _ => 0,
        }
    }).sum();

    (query_length, anchor_alignment_result.penalty)
}
fn get_base_pair_position_set(anchor_alignment_result: &AnchorAlignmentResult) -> AHashSet<(usize, usize)> {
    let mut position_set = AHashSet::new();

    let alignment_position = &anchor_alignment_result.position;
    let mut record_index = alignment_position.record.0;
    let mut query_index = alignment_position.query.0;

    for op in &anchor_alignment_result.operations {
        let count = op.count as usize;

        match op.case {
            AlignmentCase::Match | AlignmentCase::Subst=> {
                for v in (record_index..record_index+count).zip(query_index..query_index+count) {
                    position_set.insert(v);
                }

                record_index += count;
                query_index += count;
            },
            AlignmentCase::Insertion => {
                record_index += count;
            },
            AlignmentCase::Deletion => {
                query_index += count;
            },
        }
    }

    position_set
}