use ahash::AHashSet;
use sigalign::results::Alignment;

use super::{
    DpMatrix,
    parse_the_unoverlapped_alignments_with_path,
};

const PREC_SCALE: u32 = 100_000;

pub fn parse_valid_local_result_from_dpm(
    dp_matrix: &DpMatrix,
    minimum_length: u32,
    maximum_penalty_per_length: f32,
) -> Vec<Alignment> {
    let mut result = Vec::new();
    let mut paths = AHashSet::new();

    let query_length = dp_matrix.query.len();
    for substring_length in (1..=query_length).rev() {
        for start_query_index in 0..(query_length+1-substring_length) {
            let last_query_index = start_query_index + substring_length - 1;

            let unoverlapped_alignments_with_path = parse_the_unoverlapped_alignments_with_path(
                dp_matrix,
                start_query_index,
                last_query_index,
            );

            for (alignment, path) in unoverlapped_alignments_with_path.into_iter() {
                let length = alignment.length;
                let penalty = alignment.penalty;
                if (
                    length >= minimum_length
                ) && (
                    penalty * PREC_SCALE <= (length * (maximum_penalty_per_length * PREC_SCALE as f32) as u32)
                ) {
                    if paths.is_disjoint(&path) {
                        result.push(alignment);
                    }
                    paths.extend(&path);
                }
            }
        }
    }

    result
}
