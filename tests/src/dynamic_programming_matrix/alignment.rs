use super::DpMatrix;
use sigalign::{
    results::{
        AlignmentResult,
        TargetAlignmentResult,
    },
    utils::FastaReader,
};
use std::path::PathBuf;

impl DpMatrix {
    pub fn semi_global_alignment_query(
        query: &[u8],
        ref_file: &PathBuf,
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        min_length: u32,
        max_penalty_per_length: f32,
    ) -> AlignmentResult {
        let ref_reader = FastaReader::from_path(ref_file).unwrap();
        let result = ref_reader.into_iter().enumerate().filter_map(|(index, (_, target))| {
            let dp_matrix = Self::new(
                query.to_vec(),
                target,
                mismatch_penalty,
                gap_open_penalty,
                gap_extend_penalty,
            );
            let alignments = dp_matrix.parse_the_valid_semi_global_result(min_length, max_penalty_per_length);

            if alignments.len() != 0 {
                Some(TargetAlignmentResult {
                    index: index as u32,
                    alignments,
                })
            } else {
                None
            }
        }).collect();

        AlignmentResult(result)
    }
}