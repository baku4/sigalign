use crate::{Result, error_msg};
use super::{Reference, SequenceProvider, Labeling, Writable, FastaReader, reverse_complement_of_nucleotide_sequence};

use serde::{Serialize, Deserialize};

use std::path::Path;

const DEFAULT_LABEL: &str = "Reference";

/// Basic implementation for [SequenceProvider] storing sequences in-memory.
#[derive(Debug, Deserialize, Serialize)]
pub struct InMemoryProvider {
    records: Vec<SequenceRecord>,
}

impl SequenceProvider for InMemoryProvider {
    fn total_record_count(&self) -> usize {
        self.records.len()
    }
    fn sequence_of_record(&mut self, record_index: usize) -> &[u8] {
        self.records[record_index].sequence()
    }
}

impl Labeling for InMemoryProvider {
    fn label_of_record(&mut self, record_index: usize) -> &str {
        self.records[record_index].label()
    }
}

impl Writable for InMemoryProvider {}

impl InMemoryProvider {
    pub fn new_empty() -> Self {
        Self {
            records: Vec::new(),
        }
    }
    pub fn add_labeled_sequence(&mut self, label: String, sequence: Vec<u8>) {
        let sequence_record = SequenceRecord::new_forward(label, sequence);

        self.records.push(sequence_record);
    }
    pub fn add_labeled_sequence_of_nucleotide_with_reverse_complement(&mut self, label: String, sequence: Vec<u8>) {
        let reverse_complement_sequence = reverse_complement_of_nucleotide_sequence(&sequence);

        let sequence_record_forward = SequenceRecord::new_forward(
            label.clone(),
            sequence,
        );
        let sequence_record_reverse = SequenceRecord::new_reverse(
            label,
            reverse_complement_sequence,
        );

        self.records.push(sequence_record_forward);
        self.records.push(sequence_record_reverse);
    }
    pub fn from_one_sequence(sequence: Vec<u8>) -> Self {
        let mut in_memory_provider = Self::new_empty();
        in_memory_provider.add_labeled_sequence(DEFAULT_LABEL.to_string(), sequence);

        in_memory_provider
    }
    pub fn from_one_sequence_of_nucleotide_with_reverse_complement(sequence: Vec<u8>) -> Self {
        let mut in_memory_provider = Self::new_empty();
        in_memory_provider.add_labeled_sequence_of_nucleotide_with_reverse_complement(DEFAULT_LABEL.to_string(), sequence);

        in_memory_provider
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
    pub fn from_fasta_bytes(
        fasta_bytes: &[u8]
    ) -> Self {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);

        let records = fasta_reader.into_iter().map(|(label, sequence)| {
            SequenceRecord::new_forward(label, sequence)
        }).collect();

        Self {
            records
        }
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

        (record.label().to_string(), record.is_forward())
    }
}

impl Writable for Reference<InMemoryProvider> {}

#[derive(Debug, Deserialize, Serialize)]
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
    fn label(&self) -> &str {
        &self.label
    }
    fn is_forward(&self) -> bool {
        match self.direction {
            Direction::Forward => true,
            Direction::Reverse => false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
enum Direction {
    Forward,
    Reverse,
}