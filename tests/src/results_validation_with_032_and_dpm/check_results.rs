use std::cmp::Ordering;

use log::{error, info};
use sigalign::results::{Alignment, QueryAlignment, TargetAlignment};
use ahash::{
    AHashSet as HashSet,
    AHashMap as HashMap,
};

pub fn compare_the_results_are_the_same_and_return_errored_target_index(
    left: &QueryAlignment,
    right: &QueryAlignment,
) -> Vec<u32> {
    let mut errored_target_index: HashSet<u32> = HashSet::new();

    let left_map = left.0.iter().map(|x| {
        (x.index, x)
    }).collect::<HashMap<_, _>>();
    let right_map = right.0.iter().map(|x| {
        (x.index, x)
    }).collect::<HashMap<_, _>>();

    // (1) Check if right has the valid results not in left
    //  Only in left is not needed to check - they are checked in `result_validation_only_with_cutoff`
    let only_in_right_target_index = right_map.keys().filter(|x| {
        !left_map.contains_key(x)
    }).collect::<Vec<_>>();
    errored_target_index.extend(only_in_right_target_index.iter().cloned());

    // (2) Compare the common results
    let common_target_index = right_map.keys().filter(|x| {
        left_map.contains_key(x)
    }).collect::<Vec<_>>();
    for target_index in common_target_index.iter() {
        let left_target_alignment = *left_map.get(target_index).unwrap();
        let right_target_alignment = *right_map.get(target_index).unwrap();
        
        if !is_the_same_alignments_when_deduplicated(
            left_target_alignment.clone(),
            right_target_alignment.clone(),
        ) {
            errored_target_index.insert(**target_index);
        }
    }

    errored_target_index.into_iter().collect()
}

fn is_the_same_alignments_when_deduplicated(
    left: TargetAlignment,
    right: TargetAlignment,
) -> bool {
    let mut dedup_left = left.deduplicated();
    let mut dedup_right = right.deduplicated();

    if dedup_left.alignments.len() != dedup_right.alignments.len() {
        return false;
    } else {
        sort_alignments_by_optimality(&mut dedup_left.alignments);
        sort_alignments_by_optimality(&mut dedup_right.alignments);
        dedup_left.alignments.iter().zip(dedup_right.alignments.iter()).all(|(a, b)| {
            a == b
        })
    }
}
pub fn sort_alignments_by_optimality(vec: &mut Vec<Alignment>) {
    vec.sort_by(|a, b| optimal_alignment(a, b));
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
