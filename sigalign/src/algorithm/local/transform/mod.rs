use crate::results::AlignmentOperation;

use super::{
    LocalExtension,
    AnchorAlignmentResult,
    AlignmentOperations,
};

#[inline]
pub fn transform_extension_to_result(
    extension: &LocalExtension,
    operations_buffer: &Vec<AlignmentOperations>,
) -> AnchorAlignmentResult {
    let left_operations = &operations_buffer[
        extension.left_side_operation_range.0 as usize
        ..extension.left_side_operation_range.1 as usize
    ];
    let right_operations = &operations_buffer[
        extension.right_side_operation_range.0 as usize
        ..extension.right_side_operation_range.1 as usize
    ];
    let mut operations = Vec::with_capacity(
        left_operations.len() + right_operations.len()
    );

    operations.extend_from_slice(left_operations);
    let last_index_of_right_operation = if let Some(AlignmentOperations {
        operation: AlignmentOperation::Match,
        count,
    }) = operations.last_mut() {
        let v = unsafe { right_operations.last().unwrap_unchecked() }.count;
        *count += v;
        right_operations.len() - 1
    } else {
        right_operations.len()
    };
    right_operations[..last_index_of_right_operation].iter().rev().for_each(|x| {
        operations.push(x.clone())
    });

    AnchorAlignmentResult {
        penalty: extension.penalty,
        length: extension.length,
        position: extension.alignment_position.clone(),
        operations,
    }
}