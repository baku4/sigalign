use super::{DpMatrix, BacktraceMarker};
use ahash::AHashSet;
use sigalign::results::{AlignmentOperation, AlignmentOperations, AlignmentPosition, Alignment};

pub fn parse_the_unoverlapped_alignments_with_path(
    dp_matrix: &DpMatrix,
    start_query_index: usize,
    last_query_index: usize,
) -> Vec<(Alignment, AHashSet<(usize, usize)>)> {
    // (1) Get all alignments with path
    let mut alignments_with_path = parse_the_alignments_with_path(
        dp_matrix, start_query_index, last_query_index,
    );

    // (2) Sort by
    //   - penalty
    //   - last query index
    alignments_with_path.sort_by(|(a, _), (b, _)| {
        a.penalty.cmp(&b.penalty).then( // lesser penalty: more opt
            b.position.query.1.cmp(&a.position.query.1) // larger last query index: more opt
        )
    });
    
    // (3) Deduplicates
    let mut paths = AHashSet::new();
    let mut unoverlap_alignments = Vec::new();
    for (alignment, path) in alignments_with_path.into_iter() {
        if path.is_disjoint(&paths) {
            unoverlap_alignments.push((alignment, path.clone()));
        }

        paths.extend(&path);
    }

    unoverlap_alignments
}

fn parse_the_alignments_with_path(
    dp_matrix: &DpMatrix,
    start_query_index: usize,
    last_query_index: usize,
) -> Vec<(Alignment, AHashSet<(usize, usize)>)> {
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
            Alignment {
                penalty,
                length,
                position,
                operations,
            },
            path,
        ))
    });
    alignments
}

pub fn parse_the_unique_alignments_and_its_path(
    dp_matrix: &DpMatrix,
    start_query_index: usize,
    last_query_index: usize,
) -> Vec<(Alignment, AHashSet<(usize, usize)>)> {
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
                Alignment {
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
fn backtrace_from_the_indices(
    dp_matrix: &DpMatrix,
    start_query_index: usize,
    query_index: usize,
    target_index: usize,
) -> (Vec<AlignmentOperation>, AHashSet<(usize, usize)>, u32) {
    let mut reversed_operation: Vec<AlignmentOperation> = Vec::new();
    let mut path = AHashSet::new();

    let mut i = query_index + 1;
    let mut j = target_index + 1;

    let last_penalty = dp_matrix.dp_mat[i][j].penalty;
    let mut cell_type = BacktraceMarker::FromDiag;

    while i > start_query_index && j > 0 {
        match cell_type {
            BacktraceMarker::FromDiag => {
                let btm = dp_matrix.dp_mat[i][j].btm;
                match btm {
                    BacktraceMarker::FromDiag => {
                        if dp_matrix.query[i-1] == dp_matrix.target[j-1] {
                            reversed_operation.push(AlignmentOperation::Match);
                        } else {
                            reversed_operation.push(AlignmentOperation::Subst);
                        }
                        path.insert((i, j));
                        i -= 1;
                        j -= 1;
                    },
                    BacktraceMarker::FromIns => {
                        cell_type = BacktraceMarker::FromIns;
                    },
                    BacktraceMarker::FromDel => {
                        cell_type = BacktraceMarker::FromDel;
                    },
                }
            },
            BacktraceMarker::FromIns => {
                let btm = dp_matrix.ins_mat[i][j].btm;
                reversed_operation.push(AlignmentOperation::Insertion);
                i -= 1;

                match btm {
                    BacktraceMarker::FromDiag => {
                        cell_type = BacktraceMarker::FromDiag;
                    },
                    BacktraceMarker::FromIns => {
                        cell_type = BacktraceMarker::FromIns;
                    },
                    _ => panic!(""),
                }
            },
            BacktraceMarker::FromDel => {
                let btm = dp_matrix.del_mat[i][j].btm;
                reversed_operation.push(AlignmentOperation::Deletion);
                j -= 1;

                match btm {
                    BacktraceMarker::FromDiag => {
                        cell_type = BacktraceMarker::FromDiag;
                    },
                    BacktraceMarker::FromDel => {
                        cell_type = BacktraceMarker::FromDel;
                    },
                    _ => unreachable!(""),
                }
            },
        }
    }

    let start_penalty = dp_matrix.dp_mat[i][j].penalty;

    (reversed_operation, path, last_penalty - start_penalty)
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
            AlignmentOperation::Insertion => {
                query_length += count;
            },
            AlignmentOperation::Deletion => {
                target_length += count;
            },
        }
    });

    AlignmentPosition {
        query: (last_query_index as u32 + 1 - query_length, last_query_index as u32 + 1),
        target: (last_target_index as u32 + 1 - target_length, last_target_index as u32 + 1),
    }
}
