use crate::results::{
    AlignmentOperations, AlignmentOperation,
};

impl AlignmentOperations {
    #[inline]
    // About order
    //  - The left operations is already ordered because it is extended from right.
    //  - The right operations is needed to be reversed.
    pub fn concatenate_operations(
        left_reversed_operations: &Vec<Self>,
        right_reversed_operations: &mut Vec<Self>,
        anchor_size: u32
    ) -> Vec<Self> {
        let length = left_reversed_operations.len()
            + right_reversed_operations.len()
            + anchor_size as usize;
        let mut operations = Vec::with_capacity(length);
        operations.extend_from_slice(left_reversed_operations);

        // Add anchor sized Match operation to left operations
        if let Some(
            AlignmentOperations {
                operation: AlignmentOperation::Match,
                count,
            }
        ) = operations.last_mut() {
            *count += anchor_size;
        } else {
            operations.push(
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
        ) = right_reversed_operations.last_mut() {
            if let AlignmentOperations {
                operation: AlignmentOperation::Match,
                count: left_count,
            } = unsafe { operations.last_mut().unwrap_unchecked() } {
                *left_count += *right_count;
            }
            right_reversed_operations.pop();
        };

        // TODO: Not to apply reverse
        right_reversed_operations.reverse();
        operations.extend_from_slice(right_reversed_operations);

        operations
    }
    pub fn concatenate_operations_consuming(
        mut left_reversed_operations: Vec<Self>,
        mut right_reversed_operations: Vec<Self>,
        anchor_size: u32
    ) -> Vec<Self> {
        right_reversed_operations.reverse();

        // Add anchor sized Match operation to left operations
        if let Some(
            AlignmentOperations {
                operation: AlignmentOperation::Match,
                count,
            }
        ) = left_reversed_operations.last_mut() {
            *count += anchor_size;
        } else {
            left_reversed_operations.push(
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
        ) = right_reversed_operations.first_mut() {
            if let AlignmentOperations {
                operation: AlignmentOperation::Match,
                count: left_count,
            } = left_reversed_operations.last_mut().unwrap() {
                *left_count += *right_count;
            }
            right_reversed_operations.remove(0);
        };

        left_reversed_operations.append(&mut right_reversed_operations);

        left_reversed_operations
    }
}
