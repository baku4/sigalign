use super::*;

impl DpMatrix {
    pub fn exact_local_alignment_query(
        query: &[u8],
        ref_file: &PathBuf,
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        min_length: u32,
        max_penalty_per_length: f32,
    ) -> AlignmentResult {
        let ref_reader = FastaReader::from_path(ref_file).unwrap();
        let mut target_alignment_results = Vec::new();

        ref_reader.into_iter().enumerate().for_each(|(index, (_, target))| {
            let mut target_alignment_result: TargetAlignmentResult = TargetAlignmentResult {
                index: index as u32,
                alignments: Vec::new(),
            };
            let mut paths: AHashSet<(u32, u32)> = AHashSet::new();

            let query_length = query.len();
            for substring_length in (1..=query_length).rev() {
                for query_start_index in 0..(query_length+1-substring_length) {
                    let query_last_index = query_start_index + substring_length;
                    let substring = query[query_start_index..query_last_index].to_vec();
                    let dp_matrix = Self::new(
                        substring,
                        target.clone(),
                        mismatch_penalty,
                        gap_open_penalty,
                        gap_extend_penalty,
                    );

                    let mut anchor_alignment_results = dp_matrix.parse_valid_semi_global_result(min_length, max_penalty_per_length);

                    adjust_position_of_alignments(
                        &mut anchor_alignment_results,
                        query_start_index,
                    );

                    for anchor_alignment_result in anchor_alignment_results {
                        let current_paths: AHashSet<(u32, u32)> = get_alignment_paths(&anchor_alignment_result);

                        if paths.is_disjoint(&current_paths) {
                            paths.extend(&current_paths);
                            target_alignment_result.alignments.push(anchor_alignment_result);
                        }
                    }
                }
            };

            if target_alignment_result.alignments.len() != 0 {
                target_alignment_results.push(target_alignment_result)
            }
        });

        AlignmentResult(target_alignment_results)
    }
}


fn adjust_position_of_alignments(
    anchor_alignment_results: &mut Vec<AnchorAlignmentResult>,
    query_start_index: usize,
) {
    anchor_alignment_results.iter_mut().for_each(|result| {
        let query_position = &mut result.position.query;
        query_position.0 += query_start_index as u32;
        query_position.1 += query_start_index as u32;
    });
}
fn get_alignment_paths(
    anchor_alignment_result: &AnchorAlignmentResult,
) -> AHashSet<(u32, u32)> {
    let (mut query_index, mut target_index) = {
        let query_index = anchor_alignment_result.position.query.0;
        let target_index = anchor_alignment_result.position.target.0;
        (query_index, target_index)
    };
    let mut paths = AHashSet::new();
    anchor_alignment_result.operations.iter().for_each(|operation| {
        match operation.operation {
            AlignmentOperation::Match | AlignmentOperation::Subst => {
                for _ in 0..operation.count {
                    paths.insert((query_index, target_index));
                    query_index += 1;
                    target_index += 1;
                }
            },
            AlignmentOperation::Insertion => {
                target_index += operation.count;
            },
            AlignmentOperation::Deletion => {
                query_index += operation.count;
            },
        }
    });
    paths
}