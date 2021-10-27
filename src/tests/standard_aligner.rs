use std::path::Path;
use super::*;

mod alignment;
use alignment::{
    semi_global_alignment_with_position,
    local_alignment_with_position,
};

use lt_fm_index::{FmIndex, LtFmIndexAll, LtFmIndexConfig};

#[derive(Debug)]
pub struct StandardAligner {
    mismatch_penalty: usize,
    gap_open_penalty: usize,
    gap_extend_penalty: usize,
    minimum_aligned_length: usize,
    penalty_per_scale: usize,
    pattern_size: usize,
}

impl StandardAligner {
    pub fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        penalty_per_length: f32,
    ) -> Self {
        let aligner = Aligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length).unwrap();

        let gcd = aligner.gcd;

        let mismatch_penalty = aligner.penalties.x * gcd;
        let gap_open_penalty = aligner.penalties.o * gcd;
        let gap_extend_penalty = aligner.penalties.e * gcd;
        let minimum_aligned_length = minimum_aligned_length;
        let penalty_per_scale = (PRECISION_SCALE as f32 * penalty_per_length) as usize;
        let pattern_size = aligner.kmer;

        Self {
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            minimum_aligned_length,
            penalty_per_scale,
            pattern_size,
        }
    }
    pub fn semi_global_alignment_raw(
        &self,
        standard_reference: &StandardReference,
        query: Sequence,
    ) -> AlignmentResultsByRecordIndex {
        standard_reference.semi_global_alignment_results(
            query,
            self.mismatch_penalty,
            self.gap_open_penalty,
            self.gap_extend_penalty,
            self.minimum_aligned_length,
            self.penalty_per_scale,
            self.pattern_size
        )
    }
    pub fn local_alignment_raw(
        &self,
        standard_reference: &StandardReference,
        query: Sequence,
    ) -> AlignmentResultsByRecordIndex {
        standard_reference.local_alignment_results(
            query,
            self.mismatch_penalty,
            self.gap_open_penalty,
            self.gap_extend_penalty,
            self.minimum_aligned_length,
            self.penalty_per_scale,
            self.pattern_size
        )
    }
}

pub struct StandardReference {
    total_record_count: usize,
    records: Vec<StandardRecord>,
}

impl StandardReference {
    pub fn new_from_fasta<P: AsRef<Path> + std::fmt::Debug>(
        sequence_type: SequenceType,
        fasta_path: P,
    ) -> Self {
        let allowed_sequence_type = sequence_type.allowed_type();

        let fasta_records = FastaReader::from_file_path(fasta_path).unwrap();

        let mut total_record_count = 0;
        let mut records = Vec::new();

        for (label, sequence) in fasta_records {
            let record = StandardRecord::new(
                &allowed_sequence_type,
                label,
                sequence
            );

            total_record_count += 1;
            records.push(record);
        }

        Self {
            total_record_count,
            records,
        }
    }
    fn semi_global_alignment_results(
        &self,
        query: Sequence,
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        penalty_per_scale: usize,
        pattern_size: usize,
    ) -> AlignmentResultsByRecordIndex {        
        AlignmentResultsByRecordIndex(
            self.records.iter().enumerate().filter_map(|(record_index, standard_record)| {
                let alignment_results = standard_record.semi_global_alignment_results(query, mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_scale, pattern_size);

                if alignment_results.len() != 0 {
                    Some((record_index, alignment_results))
                } else {
                    None
                }
            }).collect()
        )
    }
    fn local_alignment_results(
        &self,
        query: Sequence,
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        penalty_per_scale: usize,
        pattern_size: usize,
    ) -> AlignmentResultsByRecordIndex {        
        AlignmentResultsByRecordIndex(
            self.records.iter().enumerate().filter_map(|(record_index, standard_record)| {
                let alignment_results = standard_record.local_alignment_results(query, mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_scale, pattern_size);

                if alignment_results.len() != 0 {
                    Some((record_index, alignment_results))
                } else {
                    None
                }
            }).collect()
        )
    }
}

struct StandardRecord {
    label: String,
    sequence: Vec<u8>,
    lt_fm_index: LtFmIndexAll,
}
impl StandardRecord {
    fn new(
        allowed_sequence_type: &AllowedSequenceType,
        label: String,
        sequence: Vec<u8>,
    ) -> Self {
        let lt_fm_index_config = match allowed_sequence_type {
            AllowedSequenceType::NucleotideOnly => {
                LtFmIndexConfig::for_nucleotide()
            },
            AllowedSequenceType::NucleotideWithNoise => {
                LtFmIndexConfig::for_nucleotide().with_noise()
            },
            AllowedSequenceType::AminoacidOnly => {
                LtFmIndexConfig::for_aminoacid()
            },
            AllowedSequenceType::AminoacidWithNoise => {
                LtFmIndexConfig::for_aminoacid().with_noise()
            },
        };

        let lt_fm_index = lt_fm_index_config.change_kmer_size(3).unwrap().generate(sequence.clone()).unwrap();

        Self {
            label,
            sequence,
            lt_fm_index,
        }
    }
    fn semi_global_alignment_results(
        &self,
        query: Sequence,
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        penalty_per_scale: usize,
        pattern_size: usize,
    ) -> Vec<AlignmentResult> {
        let query_length = query.len();
        let record_length = self.sequence.len();

        let pattern_count = query_length / pattern_size;

        let mut alignment_results = Vec::new();
        let mut used_alignment_results: HashSet<(usize, AlignmentPosition)> = HashSet::new();

        let mut anchors: Vec<StandardAnchor> = Vec::new();

        for pattern_index in 0..pattern_count {
            let query_start_position = pattern_index * pattern_size;
            let query_end_position = query_start_position + pattern_size;

            let pattern = &query[query_start_position..query_end_position];

            let positions_in_record = self.locate_pattern(pattern);

            'position_loop: for position in positions_in_record {
                for anchor_index in 0..anchors.len() {
                    let previous_anchor = &mut anchors[anchor_index];
                    let pattern_size_of_anchor = previous_anchor.pattern_size;
                    if previous_anchor.record_start_position + pattern_size_of_anchor == position as usize
                    && previous_anchor.query_start_position + pattern_size_of_anchor == pattern_index * pattern_size {
                        // extend pre anchor
                        previous_anchor.pattern_size += pattern_size;

                        continue 'position_loop;
                    }
                }

                anchors.push(
                    StandardAnchor {
                        record_start_position: position as usize,
                        query_start_position: pattern_index * pattern_size,
                        pattern_size: pattern_size,
                    }
                );
            }
        }

        for anchor in anchors {
            let optional_alignment_result = semi_global_alignment_with_position(
                &self.sequence,
                query,
                anchor.record_start_position,
                anchor.query_start_position,
                anchor.pattern_size,
                mismatch_penalty,
                gap_open_penalty,
                gap_extend_penalty,
                minimum_aligned_length,
                penalty_per_scale,
            );

            if let Some(alignment_result) = optional_alignment_result {
                let hashable_symbol_of_alignment_result = (
                    alignment_result.penalty,
                    alignment_result.position.clone()
                );
                let new_alignment_result = used_alignment_results.insert(hashable_symbol_of_alignment_result);

                if new_alignment_result {
                    alignment_results.push(alignment_result);
                }
            }
        }

        alignment_results
    }
    fn local_alignment_results(
        &self,
        query: Sequence,
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        penalty_per_scale: usize,
        pattern_size: usize,
    ) -> Vec<AlignmentResult> {
        let query_length = query.len();
        let record_length = self.sequence.len();

        let pattern_count = query_length / pattern_size;

        let mut alignment_results = Vec::new();
        let mut used_alignment_results: HashSet<(usize, AlignmentPosition)> = HashSet::new();

        let mut anchors: Vec<StandardAnchor> = Vec::new();

        for pattern_index in 0..pattern_count {
            let query_start_position = pattern_index * pattern_size;
            let query_end_position = query_start_position + pattern_size;

            let pattern = &query[query_start_position..query_end_position];

            let positions_in_record = self.locate_pattern(pattern);

            'position_loop: for position in positions_in_record {
                for anchor_index in 0..anchors.len() {
                    let previous_anchor = &mut anchors[anchor_index];
                    let pattern_size_of_previous_anchor = previous_anchor.pattern_size;
                    if previous_anchor.record_start_position + pattern_size_of_previous_anchor == position as usize
                    && previous_anchor.query_start_position + pattern_size_of_previous_anchor == pattern_index * pattern_size {
                        // extend pre anchor
                        previous_anchor.pattern_size += pattern_size;

                        continue 'position_loop;
                    }
                }

                anchors.push(
                    StandardAnchor {
                        record_start_position: position as usize,
                        query_start_position: pattern_index * pattern_size,
                        pattern_size: pattern_size,
                    }
                );
            }
        }

        for anchor in anchors {
            let optional_alignment_result = local_alignment_with_position(
                &self.sequence,
                query,
                anchor.record_start_position,
                anchor.query_start_position,
                anchor.pattern_size,
                mismatch_penalty,
                gap_open_penalty,
                gap_extend_penalty,
                minimum_aligned_length,
                penalty_per_scale,
            );

            if let Some(alignment_result) = optional_alignment_result {
                let hashable_symbol_of_alignment_result = (
                    alignment_result.penalty,
                    alignment_result.position.clone()
                );
                let new_alignment_result = used_alignment_results.insert(hashable_symbol_of_alignment_result);

                if new_alignment_result {
                    alignment_results.push(alignment_result);
                }
            }
        }

        alignment_results
    }
    fn locate_pattern(&self, pattern: Sequence) -> Vec<u64> {
        self.lt_fm_index.locate(pattern)
    }
}

#[derive(Debug)]
struct StandardAnchor {
    record_start_position: usize,
    query_start_position: usize,
    pattern_size: usize,
}