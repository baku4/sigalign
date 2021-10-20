use crate::{Result, error_msg};
use super::{SequenceProvider, FastaReader, reverse_complement_of_nucleotide_sequence};

use serde::{Serialize, Deserialize};

use std::path::Path;

const DEFAULT_LABEL: &str = "Reference";

pub struct OnMemoryProvider {
    records: Vec<SequenceRecord>,
}

impl SequenceProvider for OnMemoryProvider {
    fn total_record_count(&self) -> usize {
        self.records.len()
    }
    fn sequence_of_record(&self, record_index: usize) -> &[u8] {
        self.records[record_index].sequence()
    }
}

impl OnMemoryProvider {
    pub fn from_sequence(sequence: Vec<u8>) -> Self {
        let sequence_record = SequenceRecord::new_forward(DEFAULT_LABEL.to_string(), sequence);

        Self {
            records: vec![sequence_record]
        }
    }
    pub fn from_sequence_of_nucleotide_with_reverse_complement(sequence: Vec<u8>) -> Self {
        let reverse_complement_sequence = reverse_complement_of_nucleotide_sequence(&sequence);

        let sequence_record_forward = SequenceRecord::new_forward(
            DEFAULT_LABEL.to_string(),
            sequence,
        );
        let sequence_record_reverse = SequenceRecord::new_reverse(
            DEFAULT_LABEL.to_string(),
            reverse_complement_sequence,
        );

        Self {
            records: vec![sequence_record_forward, sequence_record_reverse]
        }
    }
    pub fn from_fasta_file<P: AsRef<Path> + std::fmt::Debug>(
        file_path: P,
    ) -> Result<Self> {
        let fasta_reader = FastaReader::from_file_path(file_path)?;

        let records = fasta_reader.into_iter().map(|(label, sequence)| {
            SequenceRecord::new_forward(label, sequence)
        }).collect();

        Ok(
            Self {
                records
            }
        )
    }
    pub fn from_fasta_file_of_nucleotide_with_reverse_complement<P: AsRef<Path> + std::fmt::Debug>(
        file_path: P,
    ) -> Result<Self> {
        let fasta_reader = FastaReader::from_file_path(file_path)?;

        let mut records = Vec::new(); // TODO: Iterator can be used?

        fasta_reader.into_iter().for_each(|(label, sequence)| {
            let reverse_complement_sequence = reverse_complement_of_nucleotide_sequence(&sequence);
            records.push(SequenceRecord::new_forward(label.clone(), sequence));
            records.push(SequenceRecord::new_reverse(label, reverse_complement_sequence));
        });

        Ok(
            Self {
                records
            }
        )
    }
    pub fn label_and_is_forward_of_record(&self, record_index: usize) -> (String, bool) {
        let record = &self.records[record_index];

        (record.label(), record.is_forward())
    }
}

struct SequenceRecord {
    label: String,
    direction: Direction,
    sequence: Vec<u8>,
}

impl SequenceRecord {
    fn new_forward(label: String, sequence: Vec<u8>) -> Self {
        Self {
            label,
            direction: Direction::Forward,
            sequence,
        }
    }
    fn new_reverse(label: String, sequence: Vec<u8>) -> Self {
        Self {
            label,
            direction: Direction::Reverse,
            sequence
        }
    }
    fn sequence(&self) -> &[u8] {
        &self.sequence
    }
    fn label(&self) -> String {
        self.label.clone() 
    }
    fn is_forward(&self) -> bool {
        match self.direction {
            Direction::Forward => true,
            Direction::Reverse => false,
        }
    }
}

enum Direction {
    Forward,
    Reverse,
}