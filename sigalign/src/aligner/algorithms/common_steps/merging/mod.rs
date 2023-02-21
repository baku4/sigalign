use crate::core::results::{
    AlignmentOperations, AlignmentOperation,
};

impl AlignmentOperations {
    pub fn concatenate_operations(
        mut left_operations: Vec<Self>,
        mut right_operations: Vec<Self>,
        anchor_size: u32
    ) -> Vec<Self> {
        right_operations.reverse();

        // Add anchor sized Match operation to left operations
        if let Some(
            AlignmentOperations {
                operation: AlignmentOperation::Match,
                count,
            }
        ) = left_operations.last_mut() {
            *count += anchor_size;
        } else {
            left_operations.push(
                AlignmentOperations {
                    operation: AlignmentOperation::Match,
                    count: anchor_size,
                }
            );
        };

        // Add right operations to left operations
        if let Some(
            AlignmentOperations {
                operation: AlignmentOperation::Match,
                count: right_count,
            }
        ) = right_operations.first_mut() {
            if let AlignmentOperations {
                operation: AlignmentOperation::Match,
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
