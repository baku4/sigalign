use std::collections::HashSet;

use super::{AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};

impl AlignmentOperation {
    pub fn concatenate_operations(
        mut left_operations: Vec<Self>,
        mut right_operations: Vec<Self>,
        anchor_size: u32
    ) -> Vec<Self> {
        right_operations.reverse();

        // Add anchor sized Match operation to left operations
        if let Some(
            AlignmentOperation {
                alignment_type: AlignmentType::Match,
                count,
            }
        ) = left_operations.last_mut() {
            *count += anchor_size;
        } else {
            left_operations.push(
                AlignmentOperation {
                    alignment_type: AlignmentType::Match,
                    count: anchor_size,
                }
            );
        };

        // Add right operations to left operations
        if let Some(
            AlignmentOperation {
                alignment_type: AlignmentType::Match,
                count: right_count,
            }
        ) = right_operations.first_mut() {
            if let AlignmentOperation {
                alignment_type: AlignmentType::Match,
                count: left_count,
            } = left_operations.last_mut().unwrap() {
                *left_count += *right_count;
            }
            right_operations.remove(0);
        };

        left_operations.append(&mut right_operations);

        left_operations
    }
}

pub struct AlignmentHashSet {
    hash_set: HashSet<(usize, AlignmentPosition)>
}

impl AlignmentHashSet {
    pub fn new() -> Self {
        Self {
            hash_set: HashSet::new()
        }
    }
    pub fn insert_and_check_new(&mut self, penalty: usize, alignment_position: AlignmentPosition) -> bool {
        self.hash_set.insert((penalty, alignment_position))
    }
}