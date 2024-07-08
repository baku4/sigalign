use ahash::AHashSet;
use sigalign::results::Alignment;

use super::{
    DpMatrix,
    parse_the_unique_alignments_and_its_path,
    concat_ops,
    get_alignment_position,
};

pub fn parse_valid_local_result(
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

            let unique_alignments = parse_the_unique_alignments_and_its_path(
                dp_matrix,
                start_query_index,
                last_query_index,
            );

            unique_alignments.into_iter().for_each(|(x, path)| {
                let length = x.length;
                let penalty = x.penalty;
                if (
                    length >= minimum_length
                ) && (
                    penalty <= (length as f64 * maximum_penalty_per_length as f64) as u32
                ) {
                    // Valid
                    // And unique
                    if paths.is_disjoint(&path) {
                        paths.extend(&path);
                        result.push(x);
                    }
                }
            });
        }
    }

    result
}
