use super::{PRECISION_SCALE, Cutoff, Penalties, MinPenaltyForPattern};
use super::{ReferenceInterface, Sequence, Reference, SequenceProvider};

use super::{AnchorTable, PatternAnchors, Anchor};

use std::collections::HashMap;

// impl AnchorsByRecord {
//     pub fn new<S: SequenceProvider>(
//         reference: &Reference<S>,
//         query: Sequence,
//         pattern_size: usize,
//     ) -> Self {
//         let qry_len = query.len();
//         let pattern_count = qry_len / pattern_size;

//         let mut anchors_by_record: HashMap<usize, AnchorTable> = HashMap::new();

//         for pattern_index in 0..pattern_count {
//             let qry_pos = pattern_index * pattern_size;
//             let pattern = &query[qry_pos..qry_pos+pattern_size];

//             let reference_location = reference.locate(pattern);

//             for record_location in reference_location {
//                 match anchors_by_record.get_mut(&record_location.record_index) {
//                     Some(anchor_table) => {
//                         let pattern_anchors = PatternAnchors::new(record_location.sorted_positions);
//                         anchor_table.add_pattern_anchors(pattern_index, pattern_anchors);
//                     },
//                     None => {
//                         let mut new_anchors_preset = Self::new(pattern_count);
//                         new_anchors_preset.add_new_position(pattern_index, record_location.sorted_positions);
//                         anchors_by_record.insert(record_location.record_index, new_anchors_preset);
//                     }
//                 }
//             }
//         }

//         anchors_preset_by_record
//     }
// }

// impl AnchorTable {
//     fn new_empty(pattern_count: usize) -> Self {
//         Self(vec![PatternAnchors::new_empty(); pattern_count])
//     }
//     fn add_pattern_anchors(&mut self, index: usize, pattern_anchors: PatternAnchors) {
//         self.0[index] = pattern_anchors;
//     }
// }

// impl PatternAnchors {
//     fn new_empty() -> Self {
//         Self {
//             sorted_anchors: Vec::new(),
//         }
//     }
//     fn new(sorted_position: Vec<usize>) -> Self {
//         Self(sorted_position)
//     }
// }

// impl Anchor {

// }
