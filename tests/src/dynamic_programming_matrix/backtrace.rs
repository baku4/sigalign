use super::DpMatrix;
use sigalign::results::{
    AlignmentOperations,
    AlignmentOperation, AnchorAlignmentResult, AlignmentPosition,
};
use std::cmp;
use ahash::AHashSet;

impl DpMatrix {
    pub fn parse_the_valid_semi_global_result(
        &self,
        minimum_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Vec<AnchorAlignmentResult> {
        let last_query_index = self.query.len() - 1;
        let last_target_index = self.target.len() - 1;

        // (query_index, target_index, penalty)
        let mut all_endpoints = Vec::new();
        for i in 0..self.query.len() - 1 {
            let penalty = self.get_penalty_from_the_endpoint(i, last_target_index);
            all_endpoints.push((i, last_target_index, penalty));
        }
        for j in 0..self.target.len() - 1 {
            let penalty = self.get_penalty_from_the_endpoint(last_query_index, j);
            all_endpoints.push((last_query_index, j, penalty));
        }
        {
            let penalty = self.get_penalty_from_the_endpoint(last_query_index, last_target_index);
            all_endpoints.push((last_query_index, last_target_index, penalty));
        }

        all_endpoints.sort_by_key(|x| { x.2 });

        let mut paths = AHashSet::new();
        all_endpoints.into_iter().filter_map(|(
            query_index, target_index, penalty
        )| {
            let (reversed_operation, path) = self.get_reversed_operation_from_the_endpoint(query_index, target_index);

            if paths.is_disjoint(&path) {
                paths.extend(&path);

                let length = reversed_operation.len() as u32;
                if (
                    length >= minimum_length
                ) && (
                    penalty <= (length as f64 * maximum_penalty_per_length as f64) as u32
                ) {
                    // Valid
                    let operations = concat_ops(reversed_operation);

                    let position = get_alignment_position(
                        query_index,
                        target_index,
                        &operations,
                    );

                    Some(AnchorAlignmentResult {
                        penalty,
                        length,
                        position,
                        operations,
                    })
                } else {
                    // Invalid
                    None
                }
            } else {
                None
            }

        }).collect()
    }
    fn get_penalty_from_the_endpoint(
        &self,
        query_index: usize,
        target_index: usize,
    ) -> u32 {
        let (mp, xp, yp) = self.matrix[query_index][target_index];
        cmp::min(cmp::min(mp, xp), yp)
    }
    fn get_reversed_operation_from_the_endpoint(
        &self,
        query_index: usize,
        target_index: usize,
    ) -> (Vec<AlignmentOperation>, AHashSet<(usize, usize)>) {
        let mut reversed_operation: Vec<AlignmentOperation> = Vec::new();
        let mut path = AHashSet::new();

        let mut i = query_index + 1;
        let mut j = target_index + 1;

        while i > 0 && j > 0 {
            let (m_score, x_score, y_score) = self.matrix[i][j];
            let min_score = cmp::min(cmp::min(m_score, x_score), y_score);

            if min_score == x_score {
                reversed_operation.push(AlignmentOperation::Deletion);
                i -= 1;
            } else if min_score == y_score {
                reversed_operation.push(AlignmentOperation::Insertion);
                j -= 1;
            } else { // min_score == m_score
                if self.query[i - 1] == self.target[j - 1] {
                    reversed_operation.push(AlignmentOperation::Match);
                } else {
                    reversed_operation.push(AlignmentOperation::Subst);
                }
                path.insert((i, j));
                i -= 1;
                j -= 1;
            }
        }

        (reversed_operation, path)
    }
}

fn concat_ops(
    reversed_operation: Vec<AlignmentOperation>,
) -> Vec<AlignmentOperations> {
    let mut result: Vec<AlignmentOperations> = Vec::new();

    for op in reversed_operation.into_iter().rev() {
        if let Some(AlignmentOperations {
            operation,
            count
        }) = result.last_mut() {
            if *operation == op {
                *count += 1;
                continue;
            }
        }

        result.push(AlignmentOperations {
            operation: op,
            count: 1,
        });
    }

    result
}
fn get_alignment_position(
    last_query_index: usize,
    last_target_index: usize,
    operations: &Vec<AlignmentOperations>,
) -> AlignmentPosition {
    let mut query_length = 0;
    let mut target_length = 0;

    operations.iter().for_each(|AlignmentOperations { operation, count }| {
        match operation {
            AlignmentOperation::Match | AlignmentOperation::Subst => {
                query_length += count;
                target_length += count;
            },
            AlignmentOperation::Deletion => {
                query_length += count;
            },
            AlignmentOperation::Insertion => {
                target_length += count;
            },
        }
    });

    AlignmentPosition {
        query: (last_query_index as u32 + 1 - query_length, last_query_index as u32 + 1),
        target: (last_target_index as u32 + 1 - target_length, last_target_index as u32 + 1),
    }
}
