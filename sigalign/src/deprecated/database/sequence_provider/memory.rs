use crate::deprecated::utils::get_reverse_complement;
use crate::deprecated::io::fasta::{FastaRecords, fasta_records};
use crate::deprecated::io::index::{read_lt_fm_index_from_file_path, read_lt_fm_index_from_inferred_path};
use super::{SequenceProvider, AccumulatedLength, Direction};

use serde::{Serialize, Deserialize};

const DEFAULT_STRING_LABEL: &str = "Reference";

/// [super::SequenceRecords] that stores sequences in memory.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct OnMemoryProvider {
    pub records: Vec<SequenceRecord>,
}
impl OnMemoryProvider {
    pub fn from_string(reverse_complement: bool, sequence: Vec<u8>) -> (Self, AccumulatedLength) {
        if reverse_complement {
            let seq_len = sequence.len();
            let accumulated_length = vec![(0, seq_len as u64), (seq_len as u64, (seq_len*2) as u64)];
            let rc_seq = get_reverse_complement(&sequence);
            let sequence_records: Vec<SequenceRecord> = vec![
                SequenceRecord::new(DEFAULT_STRING_LABEL, true, sequence),
                SequenceRecord::new(DEFAULT_STRING_LABEL, false, rc_seq),
                ];
            (Self{ records: sequence_records }, accumulated_length)
        } else {
            let seq_len = sequence.len();
            let accumulated_length = vec![(0, seq_len as u64)];
            let sequence_records: Vec<SequenceRecord> = vec![
                SequenceRecord::new(DEFAULT_STRING_LABEL, true, sequence),
                ];
            (Self{ records: sequence_records }, accumulated_length)
        }
    }
    pub fn from_fasta(reverse_complement: bool, fasta_path: &str) -> (Self, AccumulatedLength) {
        let mut seq_len: usize = 0;
        let mut accumulated_length: AccumulatedLength = Vec::new();
        let mut sequence_records: Vec<SequenceRecord> = Vec::new();
        // Get records of sequences
        let mut records: FastaRecords = fasta_records(fasta_path);
        while let Some(Ok(record)) = records.next() {
            let label = record.id();
            let seq = record.seq();
            if reverse_complement {
                accumulated_length.push((seq_len as u64, (seq_len + seq.len()) as u64)); //TODO: de-redundant
                seq_len += seq.len();
                accumulated_length.push((seq_len as u64, (seq_len + seq.len()) as u64));
                seq_len += seq.len();
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
                accumulated_length.push((seq_len as u64, (seq_len + seq.len()) as u64));
                seq_len += seq.len();
                sequence_records.push(
                    SequenceRecord::new(label, true, seq.to_vec())
                );
            }
        }
        (Self{ records: sequence_records }, accumulated_length)
    }
}
impl<'a> SequenceProvider<'a> for OnMemoryProvider {
    fn len(&self) -> usize {
        self.records.len()
    }
    fn sequence(&self, index: usize) -> &[u8] {
        &self.records[index].sequence
    }
    fn label(&self, index: usize) -> &str {
        &self.records[index].label
    }
    fn concated_sequence(&self) -> Vec<u8> {
        self.records.iter().map(|sr| sr.sequence.clone()).flatten().collect::<Vec<u8>>()
    }
    fn accumulated_length(&self) -> AccumulatedLength {
        let mut accumulated_length: AccumulatedLength = Vec::with_capacity(self.records.len());
        let mut total_len: usize = 0;
        self.records.iter().for_each(|sr| {
            let seq_len = sr.sequence.len();
            accumulated_length.push((total_len as u64, (total_len + seq_len) as u64));
            total_len += seq_len;
        });
        accumulated_length
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SequenceRecord {
    pub label: String,
    pub direction: Direction,
    pub sequence: Vec<u8>,
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
