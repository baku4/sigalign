use std::path::Path;
use super::*;

mod alignment;
use alignment::semi_global_alignment_with_position;

use lt_fm_index::{FmIndex, LtFmIndexAll, LtFmIndexConfig};

use bio::io::{fasta, fastq};
use bio::io::fasta::IndexedReader;

struct StandardAligner {
    mismatch_penalty: usize,
    gap_open_penalty: usize,
    gap_extend_penalty: usize,
    minimum_aligned_length: usize,
    penalty_per_length: f32,
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
        let penalty_per_length = penalty_per_length * gcd as f32;
        let pattern_size = aligner.kmer;

        Self {
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            minimum_aligned_length,
            penalty_per_length,
            pattern_size,
        }
    }
    pub fn semi_global_alignment_raw(
        &self,
        sample_reference: &StandardReference,
        query: Sequence,
    ) -> AlignmentResultsByRecordIndex {
        sample_reference.semi_global_alignment_results(
            query,
            self.mismatch_penalty,
            self.gap_open_penalty,
            self.gap_extend_penalty,
            self.minimum_aligned_length,
            self.penalty_per_length,
            self.pattern_size
        )
    }
}

struct StandardReference {
    total_record_count: usize,
    records: Vec<StandardRecord>,
}

impl StandardReference {
    fn new_from_fasta<P: AsRef<Path> + std::fmt::Debug>(
        allowed_sequence_type: &AllowedSequenceType,
        fasta_path: P,
    ) -> Self {
        let mut fasta_records = fasta::Reader::from_file(fasta_path).unwrap().records();

        let mut total_record_count = 0;
        let mut records = Vec::new(); 

        while let Some(Ok(record)) = fasta_records.next() {
            let label = record.id().to_string();
            let sequence = record.seq().to_vec();

            let record = StandardRecord::new(
                allowed_sequence_type,
                label,
                sequence
            );

            total_record_count += 1;
            records.push(record);
        };

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
        penalty_per_length: f32,
        pattern_size: usize,
    ) -> AlignmentResultsByRecordIndex {        
        AlignmentResultsByRecordIndex(
            self.records.iter().enumerate().filter_map(|(record_index, standard_record)| {
                let alignment_results = standard_record.semi_global_alignment_results(query, mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length, pattern_size);

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
        penalty_per_length: f32,
        pattern_size: usize,
    ) -> Vec<AlignmentResult> {
        let query_length = query.len();
        let record_length = self.sequence.len();

        let pattern_count = query_length / pattern_size;

        let mut alignment_results = Vec::new();

        for pattern_index in 0..pattern_count {
            let query_start_position = pattern_index * pattern_size;
            let query_end_position = query_start_position + pattern_size;

            let pattern = &query[query_start_position..query_end_position];

            let positions_in_record = self.locate_pattern(pattern);

            for position in positions_in_record {
                let optional_alignment_result = semi_global_alignment_with_position(
                    &self.sequence,
                    query,
                    position as usize,
                    pattern_index * pattern_size,
                    pattern_size,
                    mismatch_penalty,
                    gap_open_penalty,
                    gap_extend_penalty,
                    minimum_aligned_length,
                    penalty_per_length,
                );

                if let Some(alignment_result) = optional_alignment_result {
                    alignment_results.push(alignment_result);
                }
;            }
        }

        alignment_results
    }
    fn locate_pattern(&self, pattern: Sequence) -> Vec<u64> {
        self.lt_fm_index.locate(pattern)
    }
}
