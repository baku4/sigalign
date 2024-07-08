use super::DpMatrix;
use sigalign::results::{
    QueryAlignment,
    TargetAlignment,
    Alignment,
    AlignmentOperation,
};
use sigalign_utils::sequence_reader::{
    fasta::FastaReader, SeqRecord,
};
use std::path::PathBuf;
use ahash::AHashSet;

pub fn semi_global_with_dpm(
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

        let dp_matrix = DpMatrix::new(
            query.to_vec(),
            target_buffer.clone(),
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
        );
        let alignments = dp_matrix.parse_valid_semi_global_result(min_length, max_penalty_per_length);

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
