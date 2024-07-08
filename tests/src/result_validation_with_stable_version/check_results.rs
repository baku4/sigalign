use std::cmp::Ordering;
use log::{error, info};
use ahash::AHashSet as HashSet;

use sigalign::results::{Alignment, QueryAlignment, TargetAlignment};


// Check if the left query alignment is same or more optimal than the right query alignment
pub fn is_left_query_alignment_better(
    query_index: usize,
    left: &QueryAlignment,
    right: &QueryAlignment,
) -> bool {
    let sorted_a = sort_target_alignments(&left.0);
    let sorted_b = sort_target_alignments(&right.0);

    let is_left_better = if sorted_a.len() != sorted_b.len() {
        let a_set = sorted_a.iter().map(|x| x.index).collect::<HashSet<_>>();
        let b_set = sorted_b.iter().map(|x| x.index).collect::<HashSet<_>>();
        if a_set.is_superset(&b_set) {
            // info!(
            //     "(Query index {}) Unequal target alignment count (left: {}, right: {}).\nBut left is superset of right: left have {:?}",
            //     query_index, sorted_a.len(), sorted_b.len(), a_set.difference(&b_set)
            // );
            true
        } else {
            error!(
                "(Query index {}) Unequal target alignment count (left: {}, right: {}).\nLeft is not superset of right: right have {:?}",
                query_index, sorted_a.len(), sorted_b.len(), b_set.difference(&a_set)
            );
            false
        }
    } else {
        sorted_a.into_iter().zip(sorted_b.into_iter()).all(|(a,b)| {
            is_left_target_alignment_better_after_deduplication(&a, &b)
        })
    };

    if !is_left_better {
        error!("In query index: {}", query_index);
    }
    is_left_better
}
fn sort_target_alignments(vec: &Vec<TargetAlignment>) -> Vec<TargetAlignment> {
    let mut sorted = vec.clone();
    sorted.sort_by_key(|v| v.index);
    sorted
}
fn is_left_target_alignment_better_after_deduplication(
    left: &TargetAlignment,
    right: &TargetAlignment,
) -> bool {
    let left_is_better = {
        if left.index == right.index {
            let dedup_a = left.clone().deduplicated();
            let dedup_b = right.clone().deduplicated();
            if is_left_alignment_better(&dedup_a.alignments, &dedup_b.alignments) {
                true
            } else {
                false
            }
        } else {
            false
        }
    };
    if !left_is_better {
        error!("In target index: {}", left.index);
    }
    left_is_better
}
fn is_left_alignment_better(
    left: &Vec<Alignment>,
    right: &Vec<Alignment>,
) -> bool {
    let set_a = left.iter().map(|x| x.clone()).collect::<HashSet<_>>();
    let set_b = right.iter().map(|x| x.clone()).collect::<HashSet<_>>();

    let mut only_in_a: Vec<Alignment> = set_a.difference(&set_b).into_iter()
        .map(|x| x.clone()).collect();
    let only_in_b: Vec<Alignment> = set_b.difference(&set_a).into_iter()
        .map(|x| x.clone()).collect();

    let a_is_more_optimal = if only_in_b.len() == 0 {
        true
    } else {
        let mut merged = Vec::new();
        merged.extend(left.iter().cloned());
        merged.extend(right.iter().cloned());
        let pseudo_deduplicated_target_alignment = TargetAlignment {
            index: 0,
            alignments: merged,
        }.deduplicated();
        // drop the operations for comparison
        //   - This is necessary because the operations can be slightly differ by the anchor's side
        drop_the_operations_for_alignments(&mut only_in_a);
        let optimal_alignments: HashSet<_> = {
            let mut alignments = pseudo_deduplicated_target_alignment.alignments;
            drop_the_operations_for_alignments(&mut alignments);
            alignments.into_iter().collect()
        };
        let a_is_more_optimal = only_in_a.iter().all(|x| {
            optimal_alignments.contains(x)
        });
        if a_is_more_optimal {
            true
        } else {
            false
        }
    };

    // if a_is_more_optimal {
    //     info!("It is OK, since left ({}) is more optimal than right ({})", left.len(), right.len());
    // } else {
    //     error!("Left is not optimal.\nonly in left: {:?}\nonly in right: {:?}", left, right);
    // }
    if !a_is_more_optimal {
        error!("Left is not optimal.\nonly in left: {:?}\nonly in right: {:?}", only_in_a, only_in_b);
    }
    a_is_more_optimal
}

fn drop_the_operations_for_alignments(alignments: &mut Vec<Alignment>) {
    for alignment in alignments.iter_mut() {
        alignment.operations.clear();
    }
}

fn sort_by_optimal(vec: &Vec<Alignment>) -> Vec<Alignment> {
    let mut sorted = vec.clone();
    sorted.sort_by(|a, b| optimal_alignment(a, b));
    sorted
}
fn optimal_alignment(a: &Alignment, b: &Alignment) -> Ordering {
    // (1) large length is optimal
    let order1 = b.length.cmp(&a.length);
    if let Ordering::Equal = order1 {
        // (2) small penalty is optimal
        let order2 = a.penalty.cmp(&b.penalty); 
        if let Ordering::Equal = order2 {
            // (3) small query start is optimal
            let order3 = a.position.query.0.cmp(&b.position.query.0); 
            if let Ordering::Equal = order3 {
                let order4 = a.position.query.1.cmp(&b.position.query.1);
                if let Ordering::Equal = order4 {
                    let order5 = a.position.target.0.cmp(&b.position.target.0);
                    if let Ordering::Equal = order5 {
                        a.position.target.1.cmp(&b.position.target.1)
                    } else {
                        order5
                    }
                } else {
                    order4
                }
            } else {
                order3
            }
        } else {
            order2
        }
    } else {
        order1
    }
}
