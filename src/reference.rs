pub use lt_fm_index::{FmIndexConfig, FmIndex};
use serde::{Serialize, Deserialize};

use crate::io::fasta::{FastaRecords, fasta_records};
use crate::io::index::{read_lt_fm_index_from_file_path, read_lt_fm_index_from_inferred_path}; //TODO: use inferred path
use crate::utils::get_reverse_complement;

/// Configurations for `reference`
pub struct ReferenceConfig {
    reverse_complement: bool,
    // Lt-fm-index
    klt_kmer: usize,
    sa_sampling_ratio: u64,
    only_nucleotide: bool,
}

impl ReferenceConfig {
    pub fn new() -> Self {
        // this is default
        Self {
            reverse_complement: false,
            klt_kmer: 8,
            sa_sampling_ratio: 2,
            only_nucleotide: true,
        }
    }
    pub fn search_reverse_complement(mut self, rc: bool) -> Self {
        self.reverse_complement = rc; self
    }
    pub fn set_kmer_size_for_klt(mut self, kmer: usize) -> Self {
        self.klt_kmer = kmer; self
    }
    pub fn set_sampling_ratio_for_sa(mut self, ssr: u64) -> Self {
        self.sa_sampling_ratio = ssr; self
    }
    pub fn contain_only_nucleotide(mut self, on: bool) -> Self {
        self.only_nucleotide = on; self
    }
    pub fn generate_reference_with_string(&self, sequence: Vec<u8>) -> Reference {
        Reference::new_with_string(sequence, self.reverse_complement, self.klt_kmer, self.sa_sampling_ratio, self.only_nucleotide)
    }
    pub fn generate_reference_with_fasta_file(&self, file_path: &str) -> Reference {
        Reference::new_with_fasta_file(file_path, self.reverse_complement, self.klt_kmer, self.sa_sampling_ratio, self.only_nucleotide)
    }
}

const DEFAULT_LABEL: &str = "Ref";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Reference {
    pub sequence_records: Vec<SequenceRecord>,
    pub lt_fm_index: FmIndex,
    pub accumulated_length: Vec<u64>,
    pub reverse_complement: bool,
    pub total_length: usize,
}

impl Reference {
    pub fn new_with_string(
        sequence: Vec<u8>,
        reverse_complement: bool,
        klt_kmer: usize,
        sa_sampling_ratio: u64,
        only_nucleotide: bool,
    ) -> Self {
        let (sequence_records, accumulated_length, seq_len) = Self::get_preset_from_string(reverse_complement, sequence);
        let lt_fm_index = Self::generate_fmindex(&sequence_records, seq_len, klt_kmer, sa_sampling_ratio, only_nucleotide);
        Self {
            sequence_records: sequence_records,
            accumulated_length: accumulated_length,
            reverse_complement: reverse_complement,
            lt_fm_index,
            total_length: seq_len,
        }
    }
    pub fn new_with_fasta_file(
        file_path: &str,
        reverse_complement: bool,
        klt_kmer: usize,
        sa_sampling_ratio: u64,
        only_nucleotide: bool,
    ) -> Self {
        let (sequence_records, accumulated_length, seq_len) = Self::get_preset_from_fasta(reverse_complement, file_path);
        let lt_fm_index = Self::generate_fmindex(&sequence_records, seq_len, klt_kmer, sa_sampling_ratio, only_nucleotide);
        let have_one_record = accumulated_length.len() == 1;
        Self {
            sequence_records: sequence_records,
            accumulated_length: accumulated_length,
            reverse_complement: reverse_complement,
            lt_fm_index,
            total_length: seq_len,
        }
    }
    pub fn load_from_string_and_index_file(sequence: Vec<u8>, index_path: &str, reverse_complement: bool) -> Result<Self, String> {
        let fm_index = read_lt_fm_index_from_file_path(index_path)?;
        let (sequence_records, accumulated_length, seq_len) = Self::get_preset_from_string(reverse_complement, sequence);
        Ok(
            Self {
                sequence_records: sequence_records,
                accumulated_length: accumulated_length,
                reverse_complement: reverse_complement,
                lt_fm_index: fm_index,
                total_length: seq_len,
            }
        )
    }
    pub fn load_from_fasta_and_index_file(fasta_path: &str, index_path: &str, reverse_complement: bool) -> Result<Self, String> {
        let fm_index = read_lt_fm_index_from_file_path(index_path)?;
        let (sequence_records, accumulated_length, seq_len) = Self::get_preset_from_fasta(reverse_complement, fasta_path);
        let have_one_record = accumulated_length.len() == 1;
        Ok(
            Self {
                sequence_records: sequence_records,
                accumulated_length: accumulated_length,
                reverse_complement: reverse_complement,
                lt_fm_index: fm_index,
                total_length: seq_len,
            }
        )
    }
    // TODO: Add function write to new fasta file
    pub fn load_from_reference_file(file_path: &str) -> Result<Self, String> {
        Self::read_index_from_file(file_path)
    }
    // Preset: sequence_records, accumulated_length, seq_len for Reference
    fn get_preset_from_string(reverse_complement: bool, sequence: Vec<u8>) -> (Vec<SequenceRecord>, Vec<u64>, usize) {
        if reverse_complement {
            let seq_len = sequence.len();
            let accumulated_length = vec![seq_len as u64, (seq_len*2) as u64];
            let rc_seq = get_reverse_complement(&sequence);
            let sequence_records: Vec<SequenceRecord> = vec![
                SequenceRecord::new(DEFAULT_LABEL, true, sequence),
                SequenceRecord::new(DEFAULT_LABEL, false, rc_seq),
                ];
            (sequence_records, accumulated_length, seq_len*2)
        } else {
            let seq_len = sequence.len();
            let accumulated_length = vec![seq_len as u64];
            let sequence_records: Vec<SequenceRecord> = vec![
                SequenceRecord::new(DEFAULT_LABEL, true, sequence),
                ];
            (sequence_records, accumulated_length, seq_len)
        }
    }
    fn get_preset_from_fasta(reverse_complement: bool, fasta_path: &str) -> (Vec<SequenceRecord>, Vec<u64>, usize) {
        let mut seq_len: usize = 0;
        let mut accumulated_length: Vec<u64> = Vec::new();
        let mut sequence_records: Vec<SequenceRecord> = Vec::new();
        // Get records of sequences
        let mut records: FastaRecords = fasta_records(fasta_path);
        while let Some(Ok(record)) = records.next() {
            let label = record.id();
            let seq = record.seq();
            if reverse_complement {
                seq_len += seq.len();
                accumulated_length.push(seq_len as u64);
                seq_len += seq.len();
                accumulated_length.push(seq_len as u64);
                // F
                sequence_records.push(
                    SequenceRecord::new(label, true, seq.to_vec())
                );
                // R
                let rc_seq = get_reverse_complement(seq);
                sequence_records.push(
                    SequenceRecord::new(label, false, rc_seq)
                );
            } else {
                seq_len += seq.len();
                accumulated_length.push(seq_len as u64);
                sequence_records.push(
                    SequenceRecord::new(label, true, seq.to_vec())
                );
            }
        }
        (sequence_records, accumulated_length, seq_len)
    }
    pub fn locate(&self, pattern: &[u8]) -> Vec<u64> {
        self.lt_fm_index.locate_w_klt(pattern)
    }
    fn generate_fmindex(
        sequence_records: &Vec<SequenceRecord>,
        seq_len: usize, // to generate text with known capacity
        klt_kmer: usize,
        sa_sampling_ratio: u64,
        only_nucleotide: bool,
    ) -> FmIndex {
        let text = SequenceRecord::concat(sequence_records, seq_len);
        if only_nucleotide {
            FmIndexConfig::new()
                .set_kmer_lookup_table(klt_kmer)
                .set_suffix_array_sampling_ratio(sa_sampling_ratio)
                .generate_fmindex(text)
        } else {
            FmIndexConfig::new()
                .set_kmer_lookup_table(klt_kmer)
                .set_suffix_array_sampling_ratio(sa_sampling_ratio)
                .contain_non_nucleotide()
                .generate_fmindex(text)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SequenceRecord {
    pub label: String,
    pub direction: Direction,
    pub sequence: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Direction {
    Forward,
    Reverse,
}

impl SequenceRecord {
    fn new(label: &str, is_forward: bool, sequence: Vec<u8>) -> Self {
        if is_forward {
            Self { label: label.to_string(), direction: Direction::Forward, sequence }   
        } else {
            Self { label: label.to_string(), direction: Direction::Reverse, sequence }
        }
    }
    fn concat(records: &Vec<Self>, seq_len: usize) -> Vec<u8> {
        let mut concated_seq: Vec<u8> = Vec::with_capacity(seq_len);
        records.iter().for_each(|record| {
            concated_seq.extend_from_slice(&record.sequence);
        });
        concated_seq
    }
}
