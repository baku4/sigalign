use crate::{
    core::{
        SeqLen,
        regulators::{
            Penalty, PREC_SCALE, Cutoff,
        },
    },
    results::{
        AlignmentOperation, AnchorAlignmentResult, AlignmentPosition, AlignmentOperations,
    }
};
use super::{AnchorTable, Anchor, AnchorIndex};
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker, SparePenaltyCalculator};
use ahash::AHashSet;

mod valid_position_candidate;
pub use valid_position_candidate::Vpc;
mod backtrace;
pub use backtrace::{
    BackTraceResult,
};
mod traversed;

pub struct LocalExtension {
    alignment_position: AlignmentPosition,
    penalty: u32,
    length: u32,
    side_operation_index: (u32, u32, u32),
    traversed_anchor_index: (u32, u32, u32),
}

#[inline]
pub fn extend_assuming_leftmost_anchor(
    anchor: &Anchor,
    pattern_index: &u32,
    pattern_size: &u32,
    spare_penalty_calculator: &SparePenaltyCalculator,
    target: &[u8],
    query: &[u8],
    penalties: &Penalty,
    cutoff: &Cutoff,
    // Buffers
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
    left_vpc_buffer: &mut Vec<Vpc>,
    right_vpc_buffer: &mut Vec<Vpc>,
    operations_buffer: &mut Vec<AlignmentOperations>,
    traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
) {
    // 1. Init
    // 1.1. Define the range of sequence to extend
    let pattern_count = anchor.pattern_count;
    let anchor_size = pattern_count * pattern_size;

    let left_target_end_index = anchor.target_position;
    let right_target_start_index = left_target_end_index + anchor_size;

    let left_query_end_index = pattern_index * pattern_size;
    let right_query_start_index = left_query_end_index + anchor_size;

    // 2. Extend to the right
    // 2.1. Get slices to extend
    let right_target_slice = &target[right_target_start_index as usize..];
    let right_query_slice = &query[right_query_start_index as usize..];
    // 2.2. Calculate the left spare penalty
    let right_spare_penalty = spare_penalty_calculator.get_right_spare_penalty(*pattern_index);
    // 2.3. Extend the side with wave front
    right_wave_front.align_right_to_end_point(
        right_target_slice,
        right_query_slice,
        penalties,
        right_spare_penalty,
    );
    // 2.4. Fill sorted vpc vector buffer
    right_vpc_buffer.clear();
    right_wave_front.fill_sorted_vpc_vector(
        &cutoff.maximum_scaled_penalty_per_length,
        right_vpc_buffer,
    );

    // 3. Extend to the left
    // 3.1. Get slices to extend
    let left_target_slice = &target[..left_target_end_index as usize];
    let left_query_slice = &query[..left_query_end_index as usize];
    // 3.2. Calculate the left spare penalty
    let max_scaled_penalty_delta_of_right = right_vpc_buffer[0].scaled_penalty_delta;
    let left_spare_penalty = spare_penalty_calculator.get_left_spare_penalty(
        max_scaled_penalty_delta_of_right,
        *pattern_index,
    );
    // 3.3. Extend the side with wave front
    left_wave_front.align_left_to_end_point(
        left_target_slice,
        left_query_slice,
        penalties,
        left_spare_penalty,
    );
    // 3.4. Fill sorted vpc vector buffer
    left_vpc_buffer.clear();
    left_wave_front.fill_sorted_vpc_vector(
        &cutoff.maximum_scaled_penalty_per_length,
        left_vpc_buffer,
    );
    
    // 4. Get extension
    // Find the optimal end point from vpc vector
    let (optimal_left_vpc_index, optimal_right_vpc_index) = Vpc::get_optimal_position(
        left_vpc_buffer,
        right_vpc_buffer,
        anchor_size * cutoff.maximum_scaled_penalty_per_length,
    );
    // Backtrace from the end point
    // Left
    let left_optimal_vpc = &left_vpc_buffer[optimal_left_vpc_index];
    let left_back_trace_result = left_wave_front.backtrace_of_left_side(
        left_optimal_vpc.penalty,
        *pattern_size,
        left_optimal_vpc.component_index,
        cutoff.maximum_scaled_penalty_per_length,
        penalties,
        operations_buffer,
        traversed_anchor_index_buffer,
    );
    
}


// FIXME: TO DEP


// #[derive(Debug, Clone)]
// pub struct TraversedAnchor {
//     pub penalty_delta: i64,
//     pub length: u32,
//     pub penalty: u32,
//     pub operations_start_index: u32,
//     pub operations_end_index: u32,
//     pub alternative_match_count: u32,
// }

// #[derive(Debug, Clone)]
// pub struct SideExtension {
//     pub penalty: u32,
//     pub length: u32,
//     pub insertion_count: u32,
//     pub deletion_count: u32,
//     // pub reversed_operations_and_symbol_index: u32,
//     reversed_operations: Vec<AlignmentOperations>,
//     traversed_anchors: Vec<TraversedAnchorDep>,
//     pub query_index_of_the_end: u32,
// }

// #[inline]
// pub fn extend_rightmost_anchor_to_left(
//     anchor_table: &mut AnchorTable,
//     anchor_index: &AnchorIndex,
//     spare_penalty_calculator: &DepSparePenaltyCalculator,
//     scaled_penalty_delta_assuming_leftmost_anchor: u32,
//     pattern_size: u32,
//     target: &[u8],
//     query: &[u8],
//     penalties: &Penalty,
//     cutoff: &Cutoff,
//     spare_penalty: &u32,
//     wave_front: &mut WaveFront,
//     side_extensions_buffer: &mut Vec<SideExtension>,
//     // reversed_operations_buffer: &mut Vec<AlignmentOperations>,
//     // symbols_buffer: &mut Vec<Symbol>,
//     sorted_vpc_vector_buffer: &mut Vec<Vpc>,
//     traversed_positions_buffer: &mut Vec<TraversedPositionDep>,
// ) {
//     sorted_vpc_vector_buffer.clear();

//     let (pattern_count, target_position) = {
//         let anchor = &anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize];        
//         (anchor.pattern_count, anchor.target_position)
//     };

//     // (1) Define the range of sequence to extend
//     let left_query_last_index = anchor_index.0 * pattern_size;

//     let left_target_slice = &target[..target_position as usize];
//     let left_query_slice = &query[..left_query_last_index as usize];

//     // (2) Extend the side with wave front
//     wave_front.align_left_to_end_point(
//         left_target_slice,
//         left_query_slice,
//         penalties,
//         *spare_penalty,
//     );
//     let sequence_end_is_reached = wave_front.is_reached_to_sequence_end();

//     // (3) Get valid position candidates
//     //   - Clear buffer
//     sorted_vpc_vector_buffer.clear();
//     //   - Fill buffer
//     wave_front.fill_sorted_vpc_vector(
//         &cutoff.maximum_scaled_penalty_per_length,
//         sorted_vpc_vector_buffer,
//     );
//     println!("# left_vpc_vector_len: {}", sorted_vpc_vector_buffer.len());

//     // (4) Append side extensions
//     sorted_vpc_vector_buffer.iter().for_each(|vpc| {
//         traversed_positions_buffer.clear();

//         let mut side_extension = wave_front.backtrace_of_left_side(
//             vpc.penalty,
//             pattern_size,
//             vpc.component_index,
//             cutoff.maximum_scaled_penalty_per_length,
//             penalties,
//             traversed_positions_buffer,
//         );
//         println!("# left_traversed_positions_len: {}", traversed_positions_buffer.len());

//         if traversed_positions_buffer.len() != 0{
            
//             // println!("side_extension:{:?}", side_extension);
//             let left_traversed_anchors = get_left_traversed_anchors_tagging_skip_info(
//                 anchor_table,
//                 traversed_positions_buffer,
//                 spare_penalty_calculator,
//                 scaled_penalty_delta_assuming_leftmost_anchor,
//                 anchor_index.0,
//                 pattern_count,
//                 target_position,
//                 sequence_end_is_reached,
//             );
    
//             side_extension.traversed_anchors = left_traversed_anchors;
//         }
        
//         side_extension.query_index_of_the_end = left_query_last_index + side_extension.length - side_extension.insertion_count;

//         side_extensions_buffer.push(side_extension);
//     })
// }
// #[inline]
// pub fn extend_leftmost_anchor_to_right(
//     anchor_table: &mut AnchorTable,
//     anchor_index: &AnchorIndex,
//     spare_penalty_calculator: &DepSparePenaltyCalculator,
//     pattern_size: u32,
//     target: &[u8],
//     query: &[u8],
//     penalties: &Penalty,
//     cutoff: &Cutoff,
//     spare_penalty: &u32,
//     wave_front: &mut WaveFront,
//     side_extensions_buffer: &mut Vec<SideExtension>,
//     sorted_vpc_vector_buffer: &mut Vec<Vpc>,
//     traversed_positions_buffer: &mut Vec<TraversedPositionDep>,
// ) {
//     sorted_vpc_vector_buffer.clear();

//     let (pattern_count, target_position) = {
//         let anchor = &anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize];        
//         (anchor.pattern_count, anchor.target_position)
//     };
//     let anchor_size = pattern_count * pattern_size;

//     // (1) Define the range of sequence to extend
//     let right_target_start_index = target_position + anchor_size;
//     let right_query_start_index = anchor_index.0 * pattern_size + anchor_size;

//     let right_target_slice = &target[right_target_start_index as usize..];
//     let right_query_slice = &query[right_query_start_index as usize..];

//     // (2) Extend the side with wave front
//     wave_front.align_right_to_end_point(
//         right_target_slice,
//         right_query_slice,
//         penalties,
//         *spare_penalty,
//     );
//     let sequence_end_is_reached = wave_front.is_reached_to_sequence_end();

//     // (3) Get valid position candidates
//     //   - Clear buffer
//     sorted_vpc_vector_buffer.clear();
//     //   - Fill buffer
//     wave_front.fill_sorted_vpc_vector(
//         &cutoff.maximum_scaled_penalty_per_length,
//         sorted_vpc_vector_buffer,
//     );

//     println!("# right_vpc_vector_len: {}", sorted_vpc_vector_buffer.len());
//     // (4) Append side extensions
//     sorted_vpc_vector_buffer.iter().for_each(|vpc| {
//         traversed_positions_buffer.clear();

//         let mut side_extension = wave_front.backtrace_of_right_side(
//             vpc.penalty,
//             pattern_size,
//             pattern_count,
//             vpc.component_index,
//             cutoff.maximum_scaled_penalty_per_length,
//             penalties,
//             traversed_positions_buffer,
//         );
//         println!("# right_traversed_positions_len: {}", traversed_positions_buffer.len());
        
//         let right_traversed_anchors = get_right_traversed_anchors_tagging_skip_info(
//             anchor_table,
//             traversed_positions_buffer,
//             spare_penalty_calculator,
//             anchor_index.0,
//             target_position,
//             sequence_end_is_reached,
//         );

//         side_extension.traversed_anchors = right_traversed_anchors;
//         side_extension.query_index_of_the_end = right_query_start_index + side_extension.length - side_extension.insertion_count;

//         side_extensions_buffer.push(side_extension);
//     })
// }



/*

// Checkpoint: (Query position, Target position)
//  - The last checkpoint is the start and end position of the alignment from the extension
#[derive(Debug, Clone)]
pub struct LocalExtension {
    pub anchor_size: u32,
    pub query_length: u32, // For Sorting
    pub left_extension: Extension,
    pub right_extension: Extension,
    pub left_checkpoints: Vec<(u32, u32)>,
    pub right_checkpoints: Vec<(u32, u32)>,
    pub penalty: u32, // For Sorting
}

impl PosTable {
    #[inline]
    pub fn extend_assuming_leftmost_anchor_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_left: &i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> Vec<LocalExtension> {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_target_last_index = anchor_position.target_position;
        let right_target_start_index = left_target_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_delta = anchor_size.as_i64() * cutoff.maximum_scaled_penalty_per_length as i64;

        // 
        // (2) Get right extension & VPC vector
        //
        let right_record_slice = &target[right_target_start_index as usize..];
        let right_query_slice = &query[right_query_start_index as usize..];

        let right_spare_penalty = calculate_spare_penalty(*scaled_penalty_delta_of_left, anchor_size, right_query_slice.len() as u32, right_record_slice.len() as u32, penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - scaled_penalty_delta_of_left;
        let mut right_sorted_vpc_vector = Vec::new();
        right_wave_front.fill_sorted_vpc_vector(
            &cutoff.maximum_scaled_penalty_per_length,
            &mut right_sorted_vpc_vector,
        );

        // 
        // (3) Get left extension & VPC vector
        //
        let left_record_slice = &target[..left_target_last_index as usize];
        let left_query_slice = &query[..left_query_last_index as usize];

        let right_max_scaled_penalty_delta = right_sorted_vpc_vector[0].scaled_penalty_delta as i64;
        let left_spare_penalty = calculate_spare_penalty(right_max_scaled_penalty_delta, anchor_size, left_query_slice.len() as u32, left_record_slice.len() as u32, penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - right_max_scaled_penalty_delta;
        let mut left_sorted_vpc_vector = Vec::new();
        left_wave_front.fill_sorted_vpc_vector(
            &cutoff.maximum_scaled_penalty_per_length,
            &mut left_sorted_vpc_vector,
        );

        println!("# left_vpc_len:{}", left_sorted_vpc_vector.len());
        println!("# right_vpc_len:{}", right_sorted_vpc_vector.len());

        //
        // (4) Get packaged indices of VPC vector
        // The result is sorted from left to right
        //
        let vpc_index_packages = Vpc::package_vpc_index(
            &left_sorted_vpc_vector,
            &right_sorted_vpc_vector,
            anchor_scaled_penalty_delta,
        );
        println!("# vpc_index_packages:{}", vpc_index_packages.len());

        //
        // (5) Get extension results from each Vpc index package
        //
        let local_extensions: Vec<LocalExtension> = {
            vpc_index_packages.into_iter().map(| VpcIndexPackageDep {
                left_vpc_indices,
                right_vpc_indices,
            } | {
                let left_checkpoints: Vec<(u32, u32)> = left_vpc_indices.iter().map(|vpc_index| {
                    let vpc = &left_sorted_vpc_vector[*vpc_index];
                    let (query_extended_length, target_extended_length) = left_wave_front.get_extended_length_of_endpoint(vpc.penalty, vpc.component_index);
                    let query_position = left_query_last_index - query_extended_length as u32;
                    let target_position = left_target_last_index - target_extended_length as u32;
                    (query_position, target_position)
                }).collect();
                let right_checkpoints: Vec<(u32, u32)> = right_vpc_indices.iter().map(|vpc_index| {
                    let vpc = &right_sorted_vpc_vector[*vpc_index];
                    let (query_extended_length, target_extended_length) = right_wave_front.get_extended_length_of_endpoint(vpc.penalty, vpc.component_index);
                    let query_position = right_query_start_index + query_extended_length as u32;
                    let target_position = right_target_start_index + target_extended_length as u32;
                    (query_position, target_position)
                }).collect();
                let left_vpc = &left_sorted_vpc_vector[*unsafe { left_vpc_indices.last().unwrap_unchecked() }];
                let left_extension = left_wave_front.backtrace_from_endpoint(left_vpc.penalty, left_vpc.component_index, penalties);
                let right_vpc = &right_sorted_vpc_vector[*unsafe { right_vpc_indices.last().unwrap_unchecked() }];
                let right_extension = right_wave_front.backtrace_from_endpoint(right_vpc.penalty, right_vpc.component_index, penalties);
                let penalty = left_extension.penalty + right_extension.penalty;
                LocalExtension {
                    anchor_size,
                    query_length: (
                        left_extension.length - left_extension.insertion_count
                        + right_extension.length - right_extension.insertion_count
                    ),
                    left_checkpoints,
                    right_checkpoints,
                    left_extension,
                    right_extension,
                    penalty,
                }
            }).collect()
        };

        local_extensions
    }
    #[inline]
    pub fn extend_assuming_rightmost_anchor_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_right: &i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> Vec<LocalExtension> {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_target_last_index = anchor_position.target_position;
        let right_target_start_index = left_target_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_delta = (anchor_size * cutoff.maximum_scaled_penalty_per_length) as i64;

        // 
        // (2) Get left extension & VPC vector
        //
        let left_record_slice = &target[..left_target_last_index as usize];
        let left_query_slice = &query[..left_query_last_index as usize];

        let left_spare_penalty = calculate_spare_penalty(*scaled_penalty_delta_of_right, anchor_size, left_query_slice.len() as u32, left_record_slice.len() as u32, penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - scaled_penalty_delta_of_right;
        let mut left_sorted_vpc_vector = Vec::new();
        left_wave_front.fill_sorted_vpc_vector(
            &cutoff.maximum_scaled_penalty_per_length,
            &mut left_sorted_vpc_vector,
        );

        // 
        // (3) Get right extension & VPC vector
        //
        let right_record_slice = &target[right_target_start_index as usize..];
        let right_query_slice = &query[right_query_start_index as usize..];

        let left_max_scaled_penalty_delta = left_sorted_vpc_vector[0].scaled_penalty_delta as i64;
        let right_spare_penalty = calculate_spare_penalty(left_max_scaled_penalty_delta, anchor_size, right_query_slice.len() as u32, right_record_slice.len() as u32, penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - left_max_scaled_penalty_delta;
        let mut right_sorted_vpc_vector = Vec::new();
        right_wave_front.fill_sorted_vpc_vector(
            &cutoff.maximum_scaled_penalty_per_length,
            &mut right_sorted_vpc_vector,
        );

        //
        // (4) Get packaged indices of VPC vector
        // The result is sorted from left to right
        //
        let vpc_index_packages = Vpc::package_vpc_index(
            &left_sorted_vpc_vector,
            &right_sorted_vpc_vector,
            anchor_scaled_penalty_delta,
        );
        println!("# left_vpc_len:{}", left_sorted_vpc_vector.len());
        println!("# right_vpc_len:{}", right_sorted_vpc_vector.len());
        println!("# vpc_index_packages:{}", vpc_index_packages.len());
        //
        // (5) Get extension results from each Vpc index package
        //
        let local_extensions: Vec<LocalExtension> = {
            vpc_index_packages.into_iter().map(| VpcIndexPackageDep {
                left_vpc_indices,
                right_vpc_indices,
            } | {
                let left_checkpoints: Vec<(u32, u32)> = left_vpc_indices.iter().map(|vpc_index| {
                    let vpc = &left_sorted_vpc_vector[*vpc_index];
                    let (query_extended_length, target_extended_length) = left_wave_front.get_extended_length_of_endpoint(vpc.penalty, vpc.component_index);
                    let query_position = left_query_last_index - query_extended_length as u32;
                    let target_position = left_target_last_index - target_extended_length as u32;
                    (query_position, target_position)
                }).collect();
                let right_checkpoints: Vec<(u32, u32)> = right_vpc_indices.iter().map(|vpc_index| {
                    let vpc = &right_sorted_vpc_vector[*vpc_index];
                    let (query_extended_length, target_extended_length) = right_wave_front.get_extended_length_of_endpoint(vpc.penalty, vpc.component_index);
                    let query_position = right_query_start_index + query_extended_length as u32;
                    let target_position = right_target_start_index + target_extended_length as u32;
                    (query_position, target_position)
                }).collect();
                let left_vpc = &left_sorted_vpc_vector[*unsafe { left_vpc_indices.last().unwrap_unchecked() }];
                let left_extension = left_wave_front.backtrace_from_endpoint(left_vpc.penalty, left_vpc.component_index, penalties);
                let right_vpc = &right_sorted_vpc_vector[*unsafe { right_vpc_indices.last().unwrap_unchecked() }];
                let right_extension = right_wave_front.backtrace_from_endpoint(right_vpc.penalty, right_vpc.component_index, penalties);
                let penalty = left_extension.penalty + right_extension.penalty;
                LocalExtension {
                    anchor_size,
                    query_length: (
                        left_extension.length - left_extension.insertion_count
                        + right_extension.length - right_extension.insertion_count
                    ),
                    left_checkpoints,
                    right_checkpoints,
                    left_extension,
                    right_extension,
                    penalty,
                }
            }).collect()
        };

        local_extensions
    }
}

impl LocalExtension {
    #[inline]
    pub fn is_valid(
        &self,
        minimum_length: &u32,
    ) -> bool {
        let length = self.left_extension.length + self.right_extension.length + self.anchor_size;
        length >= *minimum_length
    }
    #[inline]
    pub fn is_already_registered(
        &self,
        registered_checkpoints: &AHashSet<(u32, u32)>,
    ) -> bool {
        self.left_checkpoints.iter().any(|x| registered_checkpoints.contains(x))
        && self.right_checkpoints.iter().any(|x| registered_checkpoints.contains(x))
    }
    #[inline]
    pub fn register_checkpoints(
        &self,
        registered_checkpoints: &mut AHashSet<(u32, u32)>,
    ) {
        registered_checkpoints.extend(&self.left_checkpoints);
        registered_checkpoints.extend(&self.right_checkpoints);
    }
    #[inline]
    pub fn to_alignment_result(&mut self) -> AnchorAlignmentResult {
        // TODO: Can be calculated in the previous part?
        let length = self.left_extension.length + self.right_extension.length + self.anchor_size;
        let alignment_position = {
            let (query_start, target_start) = unsafe {
                *self.left_checkpoints.last().unwrap_unchecked()  
            };
            let (query_end, target_end) = unsafe {
                *self.right_checkpoints.last().unwrap_unchecked()  
            };
            AlignmentPosition{
                target: (target_start, target_end),
                query: (query_start, query_end),
            }
        };
        let operations = AlignmentOperations::concatenate_operations(
            &self.left_extension.reversed_operations,
            &mut self.right_extension.reversed_operations,
            self.anchor_size,
        );

        AnchorAlignmentResult {
            penalty: self.penalty,
            length,
            position: alignment_position,
            operations: operations,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalExtensionDep {
    pub left_extension: Extension,
    pub right_extension: Extension,
    pub left_traversed_anchors: Vec<TraversedAnchorDep>,
    pub right_traversed_anchors: Vec<TraversedAnchorDep>,
    pub left_scaled_penalty_deltas: Vec<i64>,
    pub right_scaled_penalty_deltas: Vec<i64>,
}

impl PosTable {
    #[inline]
    pub fn extend_right_first_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target_sequence: &[u8],
        query_sequence: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_left: i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> LocalExtensionDep {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.target_position;
        let right_record_start_index = left_record_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_delta = anchor_size.as_i64() * cutoff.maximum_scaled_penalty_per_length as i64;

        // 
        // (2) Get right extension & VPC vector
        //
        let right_record_slice = &target_sequence[right_record_start_index as usize..];
        let right_query_slice = &query_sequence[right_query_start_index as usize..];

        let right_spare_penalty = calculate_spare_penalty(scaled_penalty_delta_of_left, anchor_size, right_query_slice.len() as u32, right_record_slice.len() as u32, penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - scaled_penalty_delta_of_left;
        let mut right_sorted_vpc_vector = Vec::new();
        right_wave_front.fill_sorted_vpc_vector(
            &cutoff.maximum_scaled_penalty_per_length,
            &mut right_sorted_vpc_vector,
        );

        // 
        // (3) Get left extension & VPC vector
        //
        let left_record_slice = &target_sequence[..left_record_last_index as usize];
        let left_query_slice = &query_sequence[..left_query_last_index as usize];

        let right_max_scaled_penalty_delta = right_sorted_vpc_vector[0].scaled_penalty_delta as i64;
        let left_spare_penalty = calculate_spare_penalty(right_max_scaled_penalty_delta, anchor_size, left_query_slice.len() as u32, left_record_slice.len() as u32, penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - right_max_scaled_penalty_delta;
        let mut left_sorted_vpc_vector = Vec::new();
        left_wave_front.fill_sorted_vpc_vector(
            &cutoff.maximum_scaled_penalty_per_length,
            &mut left_sorted_vpc_vector,
        );

        //
        // (4) Get packaged indices of VPC vector
        // The result is sorted from left to right
        //
        let vpc_index_packages = Vpc::package_vpc_index(
            &left_sorted_vpc_vector,
            &right_sorted_vpc_vector,
            anchor_scaled_penalty_delta,
        );

        //
        // (5) Get extension result from each Vpc index package
        //
        // let left_traversed_anchors = Vec::new();
        // let right_traversed_anchors = Vec::new();
        // let local_extensions = {
        //     vpc_index_packages.into_iter().map(| VpcIndexPackage {
        //         left_vpc_indices,
        //         right_vpc_indices,
        //     } | {
        //         // let left_extension = left_wave_front.backtrace_from_endpoint(penalty, index_of_component, penalties)
        //     })
        // };

        let (left_vpc_index, right_vpc_index) = Vpc::get_optimal_position(&left_sorted_vpc_vector, &right_sorted_vpc_vector, anchor_scaled_penalty_delta, anchor_size);

        //
        // (5) Get extensions
        //
        let left_vpc = &left_sorted_vpc_vector[left_vpc_index];
        let right_vpc = &right_sorted_vpc_vector[right_vpc_index];

        let (left_extension, left_traversed_positions) = left_wave_front.backtrace_from_point_checking_left_traversed(left_vpc.penalty, left_vpc.component_index, penalties, pattern_size);
        let (right_extension, right_traversed_positions) = right_wave_front.backtrace_from_point_checking_right_traversed(right_vpc.penalty, right_vpc.component_index, penalties, pattern_size);

        let left_traversed_anchors = self.left_traversed_anchors(
            left_traversed_positions,
            anchor_index.0,
            left_record_last_index,
            left_extension.length,
            left_extension.penalty,
            pattern_size,
        );
        let right_traversed_anchors = self.right_traversed_anchors(
            right_traversed_positions,
            anchor_index.0,
            pattern_count,
            right_record_start_index,
            right_extension.length,
            right_extension.penalty,
            pattern_size,
        );

        //
        // (6) Scaled penalty delta
        //
        let left_scaled_penalty_deltas: Vec<i64> = get_scaled_penalty_deltas_of_vpc_vector(
            &left_extension,
            &left_sorted_vpc_vector,
            cutoff,
            penalties,
            &left_traversed_anchors,
        );

        let right_scaled_penalty_deltas: Vec<i64> = get_scaled_penalty_deltas_of_vpc_vector(
            &right_extension,
            &right_sorted_vpc_vector,
            cutoff,
            penalties,
            &right_traversed_anchors,
        );

        LocalExtensionDep {
            left_extension,
            right_extension,
            left_traversed_anchors,
            right_traversed_anchors,
            left_scaled_penalty_deltas,
            right_scaled_penalty_deltas,
        }
    }
    #[inline]
    pub fn extend_left_first_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_right: i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> LocalExtensionDep {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.target_position;
        let right_record_start_index = left_record_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_delta = (anchor_size * cutoff.maximum_scaled_penalty_per_length) as i64;

        // 
        // (2) Get left extension & VPC vector
        //
        let left_record_slice = &target[..left_record_last_index as usize];
        let left_query_slice = &query[..left_query_last_index as usize];

        let left_spare_penalty = calculate_spare_penalty(scaled_penalty_delta_of_right, anchor_size, left_query_slice.len() as u32, left_record_slice.len() as u32, penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - scaled_penalty_delta_of_right;
        let mut left_vpc_vector = Vec::new();
        left_wave_front.fill_sorted_vpc_vector(
            &cutoff.maximum_scaled_penalty_per_length,
            &mut left_vpc_vector,
        );

        // 
        // (3) Get right extension & VPC vector
        //
        let right_record_slice = &target[right_record_start_index as usize..];
        let right_query_slice = &query[right_query_start_index as usize..];

        let left_max_scaled_penalty_delta = left_vpc_vector[0].scaled_penalty_delta as i64;
        let right_spare_penalty = calculate_spare_penalty(left_max_scaled_penalty_delta, anchor_size, right_query_slice.len() as u32, right_record_slice.len() as u32, penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - left_max_scaled_penalty_delta;
        let mut right_vpc_vector = Vec::new();
        right_wave_front.fill_sorted_vpc_vector(
            &cutoff.maximum_scaled_penalty_per_length,
            &mut right_vpc_vector,
        );

        //
        // (4) Find optimal position of VPC vectors
        //
        let (left_vpc_index, right_vpc_index) = Vpc::get_optimal_position(&left_vpc_vector, &right_vpc_vector, anchor_scaled_penalty_delta, anchor_size);

        //
        // (5) Get extensions
        //
        let left_vpc = &left_vpc_vector[left_vpc_index];
        let right_vpc = &right_vpc_vector[right_vpc_index];

        let (left_extension, left_traversed_positions) = left_wave_front.backtrace_from_point_checking_left_traversed(left_vpc.penalty, left_vpc.component_index, penalties, pattern_size);
        let (right_extension, right_traversed_positions) = right_wave_front.backtrace_from_point_checking_right_traversed(right_vpc.penalty, right_vpc.component_index, penalties, pattern_size);

        let left_traversed_anchors = self.left_traversed_anchors(
            left_traversed_positions,
            anchor_index.0,
            left_record_last_index,
            left_extension.length,
            left_extension.penalty,
            pattern_size,
        );
        let right_traversed_anchors = self.right_traversed_anchors(
            right_traversed_positions,
            anchor_index.0,
            pattern_count,
            right_record_start_index,
            right_extension.length,
            right_extension.penalty,
            pattern_size,
        );

        //
        // (6) Scaled penalty delta
        //
        let left_scaled_penalty_deltas: Vec<i64> = get_scaled_penalty_deltas_of_vpc_vector(
            &left_extension,
            &left_vpc_vector,
            cutoff,
            penalties,
            &left_traversed_anchors,
        );

        let right_scaled_penalty_deltas: Vec<i64> = get_scaled_penalty_deltas_of_vpc_vector(
            &right_extension,
            &right_vpc_vector,
            cutoff,
            penalties,
            &right_traversed_anchors,
        );

        LocalExtensionDep {
            left_extension,
            right_extension,
            left_traversed_anchors,
            right_traversed_anchors,
            left_scaled_penalty_deltas,
            right_scaled_penalty_deltas,
        }
    }
}

fn get_scaled_penalty_deltas_of_vpc_vector(
    extension: &Extension,
    vpc_vector: &Vec<Vpc>,
    cutoff: &Cutoff,
    penalties: &Penalty,
    traversed_anchors: &Vec<TraversedAnchorDep>,
) -> Vec<i64> {
    let scaled_penalty_delta_of_extension = (extension.length * cutoff.maximum_scaled_penalty_per_length) as i64 - (extension.penalty * PREC_SCALE) as i64;

    let mut vpc_index_for_traversed_anchor = 0;
    let mut scaled_penalty_deltas: Vec<i64> = traversed_anchors.iter().rev().map(|traversed_anchor| {
        let length_to_traversed_start_position = extension.length - traversed_anchor.remained_length;
        let penalty_to_traversed_start_position = extension.penalty - traversed_anchor.remained_penalty;
        let min_query_length = length_to_traversed_start_position - (penalty_to_traversed_start_position / penalties.e);
        while min_query_length > vpc_vector[vpc_index_for_traversed_anchor].query_length {
            vpc_index_for_traversed_anchor += 1;
        }
        let remained_scaled_penalty_delta = (traversed_anchor.remained_length * cutoff.maximum_scaled_penalty_per_length) as i64 - (traversed_anchor.remained_penalty * PREC_SCALE) as i64;
        vpc_vector[vpc_index_for_traversed_anchor].scaled_penalty_delta + remained_scaled_penalty_delta - scaled_penalty_delta_of_extension
    }).collect();
    scaled_penalty_deltas.reverse();

    scaled_penalty_deltas
}

 */