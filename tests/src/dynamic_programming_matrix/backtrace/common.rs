use super::DpMatrix;
use ahash::AHashSet;
use sigalign::results::{AlignmentOperation, AlignmentOperations, AlignmentPosition, AnchorAlignmentResult};
use std::cmp;

pub fn parse_the_unique_alignments_and_its_path(
    dp_matrix: &DpMatrix,
    start_query_index: usize,
    last_query_index: usize,
) -> Vec<(AnchorAlignmentResult, AHashSet<(usize, usize)>)> {
    // (1) Get all endpoints
    let mut all_endpoints = Vec::new();
    let last_target_index = dp_matrix.target.len() - 1;
    for i in 0..last_query_index {
        all_endpoints.push((i, last_target_index));
    }
    for j in 0..last_target_index {
        all_endpoints.push((last_query_index, j));
    }
    all_endpoints.push((last_query_index, last_target_index));
    // (2) Get all alignments
    let mut alignments = Vec::new();
    all_endpoints.into_iter().for_each(|(query_index, target_index)| {
        let (
            reversed_operation,
            path,
            penalty,
        ) = backtrace_from_the_indices(
            dp_matrix,
            start_query_index,
            query_index,
            target_index,
        );
        let length = reversed_operation.len() as u32;
        let operations = concat_ops(reversed_operation);
        let position = get_alignment_position(query_index, target_index, &operations);

        alignments.push((
            position,
            penalty,
            length,
            operations,
            path,
        ))
    });
    // (3) Sort by
    //   - penalty
    //   - last query index
    alignments.sort_by(|a, b| {
        a.1.cmp(&b.1).then( // lesser penalty: more opt
            b.0.query.1.cmp(&a.0.query.1) // larger last query index: more opt
        )
    });
    // (4) Deduplicates
    let mut paths = AHashSet::new();
    alignments.into_iter().filter_map(|(
        position,
        penalty,
        length,
        operations,
        path,
    )| {
        if paths.is_disjoint(&path) {
            paths.extend(&path);
            Some((
                AnchorAlignmentResult {
                    penalty,
                    length,
                    position,
                    operations,
                },
                path,
            ))
        } else {
            None
        }
    }).collect()
}

// Result: reversed_operation, path, penalty
pub fn backtrace_from_the_indices(
    dp_matrix: &DpMatrix,
    start_query_index: usize,
    query_index: usize,
    target_index: usize,
) -> (Vec<AlignmentOperation>, AHashSet<(usize, usize)>, u32) {
    let mut reversed_operation: Vec<AlignmentOperation> = Vec::new();
    let mut path = AHashSet::new();

    let mut i = query_index + 1;
    let mut j = target_index + 1;

    let last_penalty = {
        let (m_score, x_score, y_score) = dp_matrix.matrix[i][j];
        let min_score = cmp::min(cmp::min(m_score, x_score), y_score);
        min_score
    };

    while i > start_query_index && j > 0 {
        let (m_score, x_score, y_score) = dp_matrix.matrix[i][j];
        let min_score = cmp::min(cmp::min(m_score, x_score), y_score);
        
        if (min_score == m_score) && (dp_matrix.query[i - 1] == dp_matrix.target[j - 1]) {
            reversed_operation.push(AlignmentOperation::Match);
            path.insert((i, j));
            i -= 1;
            j -= 1;
        } else if min_score == x_score {
            reversed_operation.push(AlignmentOperation::Deletion);
            i -= 1;
        } else if min_score == y_score {
            reversed_operation.push(AlignmentOperation::Insertion);
            j -= 1;
        } else { // (min_score == m_score) && (self.query[i - 1] != self.target[j - 1])
            reversed_operation.push(AlignmentOperation::Subst);
            path.insert((i, j));
            i -= 1;
            j -= 1;
        }
    }

    let start_penalty = {
        let (m_score, x_score, y_score) = dp_matrix.matrix[i][j];
        let min_score = cmp::min(cmp::min(m_score, x_score), y_score);
        min_score
    };

    (reversed_operation, path, last_penalty - start_penalty)
}
pub fn concat_ops(
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
pub fn get_alignment_position(
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
pub fn cal_penalty(
    operations: &Vec<AlignmentOperations>,
    px: u32,
    po: u32,
    pe: u32,
) -> u32 {
    operations.iter().map(|x| {
        match x.operation {
            AlignmentOperation::Match => {
                0
            },
            AlignmentOperation::Subst => {
                px * x.count
            },
            AlignmentOperation::Insertion | AlignmentOperation::Deletion => {
                pe * x.count + po
            },
        }
    }).sum()
}


impl DpMatrix {
    #[inline]
    pub fn get_penalty(
        &self,
        query_index: usize,
        target_index: usize,
    ) -> u32 {
        let (mp, xp, yp) = self.matrix[query_index+1][target_index+1];
        cmp::min(cmp::min(mp, xp), yp)
    }
    pub fn get_reversed_operation_from_the_endpoint(
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
            
            if (min_score == m_score) && (self.query[i - 1] == self.target[j - 1]) {
                reversed_operation.push(AlignmentOperation::Match);
                path.insert((i, j));
                i -= 1;
                j -= 1;
            } else if min_score == x_score {
                reversed_operation.push(AlignmentOperation::Deletion);
                i -= 1;
            } else if min_score == y_score {
                reversed_operation.push(AlignmentOperation::Insertion);
                j -= 1;
            } else { // (min_score == m_score) && (self.query[i - 1] != self.target[j - 1])
                reversed_operation.push(AlignmentOperation::Subst);
                path.insert((i, j));
                i -= 1;
                j -= 1;
            }
        }

        (reversed_operation, path)
    }
}
