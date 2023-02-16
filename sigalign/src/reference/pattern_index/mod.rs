
use crate::core::PatternLocation;
use super::{SequenceType};

pub trait PatternIndex {
    type Option;

    fn new(
        concatenated_sequence_with_boundaries: ConcatenatedSequenceWithBoundaries,
        sequence_type: &SequenceType,
        option: Self::Option,
    ) -> Self;
    fn locate(&self, pattern: &[u8]) -> Vec<PatternLocation>;
}

/// If there are three targets with "ATT", "CC", "GGGG", the "concatenated_sequence" is "ATTCCGGGG" and the "boundaries" is [0, 3, 5, 9].
pub struct ConcatenatedSequenceWithBoundaries {
    pub concatenated_sequence: Vec<u8>,
    pub boundaries: Vec<u64>,
}

// Implementations
mod default;

// use crate::AHashMap;

// #[derive(Clone, PartialEq, Eq)]
// pub struct PatternFinder {
//     lt_fm_index: LtFmIndex,
//     record_boundary_positions: Vec<u64>,
// }

// impl PatternFinder {
//     pub fn new(
//         joined_sequence: JoinedSequence,
//         is_nucleotide: bool,
//         with_noise: bool,
//         compress_rank_check_point_with_128: bool,
//         suffix_array_sampling_ratio: u64,
//         kmer_size_for_lookup_table: usize,
//     ) -> Result<Self> {
//         let mut lt_fm_index_builder = LtFmIndexBuilder::new()
//             .set_lookup_table_kmer_size(kmer_size_for_lookup_table)?
//             .set_suffix_array_sampling_ratio(suffix_array_sampling_ratio)?;
        
//         lt_fm_index_builder = if is_nucleotide {
//             if with_noise {
//                 lt_fm_index_builder.use_nucleotide_with_noise()
//             } else {
//                 lt_fm_index_builder.use_nucleotide_only()
//             }
//         } else {
//             if with_noise {
//                 lt_fm_index_builder.use_amino_acid_with_noise()
//             } else {
//                 lt_fm_index_builder.use_amino_acid_only()
//             }
//         };

//         lt_fm_index_builder = if compress_rank_check_point_with_128 {
//             lt_fm_index_builder.compress_bwt_128()
//         } else {
//             lt_fm_index_builder.compress_bwt_64()
//         };

//         let lt_fm_index = lt_fm_index_builder.build(joined_sequence.bytes);

//         Ok(Self {
//             lt_fm_index,
//             record_boundary_positions: joined_sequence.record_boundary_positions,
//         })
//     }
//     pub fn locate_in_record_search_range(&self, pattern: Sequence, target_record_index: &[u32]) -> Vec<PatternLocation> {
//         let sorted_locations = self.sorted_locations_of_pattern(pattern);

//         // TODO: Applying cap is valuable?
//         let mut positions_by_record: AHashMap<usize, Vec<usize>> = AHashMap::new();

//         let pattern_size = pattern.len() as u64;
//         let search_range_count = target_record_index.len();

//         let mut size;
//         let mut left;
//         let mut right;
//         let mut mid = 0;
//         let mut index;

//         for position in sorted_locations {
//             // reset
//             right = search_range_count;
//             left = mid;
//             size = right - left;
    
//             while left < right {
//                 mid = left + size / 2;
//                 index = target_record_index[mid] as usize;
                
//                 let start = self.record_boundary_positions[index];
//                 let end = self.record_boundary_positions[index + 1];

//                 if position >= end {
//                     left = mid + 1;
//                 } else if start > position {
//                     right = mid;
//                 } else {
//                     if (position + pattern_size) <= end {
//                         let ref_pos = (position - start) as usize;
//                         match positions_by_record.get_mut(&index) {
//                             Some(v) => {
//                                 v.push(ref_pos);
//                             },
//                             None => {
//                                 positions_by_record.insert(index, vec![ref_pos]);
//                             },
//                         }
//                         break;
//                     } else {
//                         break;
//                     }
//                 }
    
//                 size = right - left;
//             }
//         }
    
//         positions_by_record.into_iter().map(|(record_index, positions)| {
//             PatternLocation {
//                 target_index: record_index,
//                 positions: positions,
//             }
//         }).collect()
//     }
//     fn sorted_locations_of_pattern(&self, pattern: Sequence) -> Vec<u64> {
//         let mut locations = self.lt_fm_index.locate(pattern);
//         locations.sort();
//         locations
//     }
// }