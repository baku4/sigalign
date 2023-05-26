use sigalign::results::AnchorAlignmentResult;

use super::{
    DpMatrix,
    parse_the_unique_alignments_and_its_path,
};

pub fn parse_valid_semi_global_result(
    dp_matrix: &DpMatrix,
    minimum_length: u32,
    maximum_penalty_per_length: f32,
) -> Vec<AnchorAlignmentResult> {
    let unique_alignments = parse_the_unique_alignments_and_its_path(
        dp_matrix,
        0,
        dp_matrix.query.len() - 1,
    );

    unique_alignments.into_iter().filter_map(|(x, _)| {
        let length = x.length;
        let penalty = x.penalty;
        if (
            length >= minimum_length
        ) && (
            penalty <= (length as f64 * maximum_penalty_per_length as f64) as u32
        ) {
            Some(x)
        } else {
            None
        }
    }).collect()
}