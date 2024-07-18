use crate::results::{
    Alignment,
    AlignmentOperations,
    AlignmentOperation,
    AlignmentPosition,
};

// Preliminary form of alignment result
#[derive(Debug, Clone)]
pub struct Extension {
    pub alignment_position: AlignmentPosition,
    pub penalty: u32,
    pub length: u32,
    pub left_side_operation_range: (u32, u32),
    pub right_side_operation_range: (u32, u32),
}

impl Extension {
    #[inline]
    pub fn parse_anchor_alignment_result(
        &self,
        operations_buffer: &Vec<AlignmentOperations>,
    ) -> Alignment {
        let left_operations = &operations_buffer[
            self.left_side_operation_range.0 as usize
            ..self.left_side_operation_range.1 as usize
        ];
        let right_operations = &operations_buffer[
            self.right_side_operation_range.0 as usize
            ..self.right_side_operation_range.1 as usize
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
        operations.extend(right_operations[..last_index_of_right_operation].iter().rev().cloned());
    
        Alignment {
            penalty: self.penalty,
            length: self.length,
            position: self.alignment_position.clone(),
            operations,
        }
    }
}
