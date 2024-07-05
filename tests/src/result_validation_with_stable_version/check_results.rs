use std::cmp::Ordering;
use log::{error, info};
use ahash::AHashSet as HashSet;

use sigalign::results::{Alignment, QueryAlignment, TargetAlignment};


// Check if the results is acceptable.
pub fn is_acceptable_query_alignment(
    a: &QueryAlignment,
    b: &QueryAlignment,
) -> bool {
    let sorted_a = sort_target_alignments(&a.0);
    let sorted_b = sort_target_alignments(&b.0);

    if sorted_a.len() != sorted_b.len() {
        error!("Unequal target alignment count. left: {}, right: {}", sorted_a.len(), sorted_b.len());
        false
    } else {
        sorted_a.into_iter().zip(sorted_b.into_iter()).all(|(a,b)| {
            is_same_or_left_is_more_optimal_target_alignments_when_deduplicated(&a, &b)
        })
    }
}

fn is_same_or_left_is_more_optimal_target_alignments_when_deduplicated(
    a: &TargetAlignment,
    b: &TargetAlignment,
) -> bool {
    if a.index == b.index {
        let dedup_a = a.clone().deduplicated();
        let dedup_b = b.clone().deduplicated();

        if is_same_or_left_is_more_optimal_alignments(&dedup_a.alignments, &dedup_b.alignments) {
            true
        } else {
            error!("Un-deduplicated alignment. left: {:?}, right: {:?}", a, b);
            error!("Unequal in target index {}", a.index);
            false
        }
    } else {
        error!("Unequal target index. left: {}, right: {}", a.index, b.index);
        false
    }
}

fn sort_target_alignments(vec: &Vec<TargetAlignment>) -> Vec<TargetAlignment> {
    let mut sorted = vec.clone();
    sorted.sort_by_key(|v| v.index);
    sorted
}
fn is_same_or_left_is_more_optimal_alignments(
    a: &Vec<Alignment>,
    b: &Vec<Alignment>,
) -> bool {
    let sorted_a = sort_by_optimal(a);
    let sorted_b = sort_by_optimal(b);

    if sorted_a.len() != sorted_b.len() {
        error!("Unequal alignment count. left: {}, right: {}", sorted_a.len(), sorted_b.len());
        false
    } else {
        let mut is_equal = true;
        for (a, b) in sorted_a.iter().zip(sorted_b.iter()) {
            let is_equal_anchor_alignment_result = {
                a.penalty == b.penalty
                && a.length == b.length
                && a.position == b.position
            };
            if !is_equal_anchor_alignment_result {
                let set_a = sorted_a.iter().map(|x| x.clone()).collect::<HashSet<_>>();
                let set_b = sorted_b.iter().map(|x| x.clone()).collect::<HashSet<_>>();

                let only_in_a: Vec<Alignment> = set_a.difference(&set_b).into_iter()
                    .map(|x| x.clone()).collect();
                let only_in_b: Vec<Alignment> = set_b.difference(&set_a).into_iter()
                    .map(|x| x.clone()).collect();

                // Check is a is more optimal than b
                if only_in_a.len() == 1 && only_in_b.len() == 1 {
                    let a = only_in_a.first().unwrap();
                    let b = only_in_b.first().unwrap();

                    let a_is_more_optimal = {
                        if a.length > b.length {
                            true
                        } else if a.length == b.length {
                            if a.penalty < b.penalty {
                                true
                            } else if a.penalty == b.penalty {
                                if a.position.query.0 < b.position.query.0 {
                                    true
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };
                    if !a_is_more_optimal {
                        is_equal = false;
                        break;
                    }
                } else {
                    is_equal = false;
                    break;
                }
            }
        }
        if !is_equal {
            error!("Unequal alignment result. left: {:?}, right: {:?}", sorted_a, sorted_b);
        }
        is_equal
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
