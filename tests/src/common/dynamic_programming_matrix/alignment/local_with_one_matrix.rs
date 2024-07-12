use super::{
    DpMatrix,
    parse_valid_local_result_from_dpm,
    target_indices_having_matched_pattern,
};
use sigalign::{
    results::{
        Alignment, AlignmentOperation, QueryAlignment, TargetAlignment
    },
    Reference,
    };
use sigalign_utils::sequence_reader::{
    fasta::FastaReader, SeqRecord,
};
use std::path::PathBuf;
use ahash::AHashSet;

pub fn dp_local_to_pattern_existing_targets(
    query: &[u8],
    sig_reference: &Reference,
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> QueryAlignment {
    let mut target_alignment_results = Vec::new();
    // Fetch target indices
    let target_indices = target_indices_having_matched_pattern(
        query,
        sig_reference,
        mismatch_penalty, 
        gap_open_penalty,
        gap_extend_penalty,
        min_length,
        max_penalty_per_length,
    );
    // Align
    for target_index in target_indices {
        let target = sig_reference.get_sequence(target_index).unwrap();
        let dp_matrix = DpMatrix::new(
            query.to_vec(),
            target.to_vec(),
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
        );
        let alignments = parse_valid_local_result_from_dpm(
            &dp_matrix, min_length, max_penalty_per_length,
        );
        if alignments.len() != 0 {
            target_alignment_results.push(TargetAlignment {
                index: target_index,
                alignments,
            });
        }
    }
    QueryAlignment(target_alignment_results)
}

pub fn dp_local_to_ref_file(
    query: &[u8],
    ref_file: &PathBuf,
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> QueryAlignment {
    let mut ref_reader = FastaReader::from_path(ref_file).unwrap();
    let mut target_buffer = Vec::new();
    let mut target_index = 0;

    let mut result = Vec::new();

    while let Some(mut record) = ref_reader.next() {
        target_buffer.clear();
        record.extend_seq_buf(&mut target_buffer);

        let alignments = dp_local_to_target(
            query,
            &target_buffer,
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            min_length,
            max_penalty_per_length,
        );

        if alignments.len() != 0 {
            result.push(TargetAlignment {
                index: target_index,
                alignments,
            });
        }

        target_index += 1;
    }
    QueryAlignment(result)
}

pub fn dp_local_to_target(
    query: &[u8],
    target: &[u8],
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> Vec<Alignment> {
    let dp_matrix = DpMatrix::new(
        query.to_vec(),
        target.to_vec(),
        mismatch_penalty,
        gap_open_penalty,
        gap_extend_penalty,
    );
    let alignments = parse_valid_local_result_from_dpm(
        &dp_matrix, min_length, max_penalty_per_length,
    );

    alignments
}
