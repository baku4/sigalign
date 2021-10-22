use crate::{Result, error_msg};
use super::{SequenceProvider, Labeling, FastaReader, reverse_complement_of_nucleotide_sequence};

mod fai;

use fai::fai_bytes_and_count_of_fasta_file;

use serde::{Serialize, Deserialize};
use bio::io::fasta::{Index, IndexedReader};

use std::path::Path;
use std::fs::File;

// #[derive(Debug, Deserialize, Serialize)]
pub struct IndexedFastaProvider {
    fasta_path: String,
    record_count: usize,
    indexed_reader: IndexedReader<File>,
    sequence_buffer: Vec<u8>,
    label_buffer: String,
}

impl IndexedFastaProvider {
    pub fn new(fasta_file_path: &str) -> Result<Self> {
        let (fai_bytes, record_count) = fai_bytes_and_count_of_fasta_file(fasta_file_path)?;
        let fasta_file = File::open(fasta_file_path)?;

        let indexed_reader = IndexedReader::new(fasta_file, &*fai_bytes)?;

        let fasta_path = fasta_file_path.to_string();

        Ok(Self {
            fasta_path,
            record_count,
            indexed_reader,
            sequence_buffer: Vec::new(),
            label_buffer: String::new(),
        })
    }
    fn clear_sequence_buffer(&mut self) {
        self.sequence_buffer.clear();
    }
}

impl SequenceProvider for IndexedFastaProvider {
    fn total_record_count(&self) -> usize {
        self.record_count
    }
    fn sequence_of_record(&mut self, record_index: usize) -> &[u8] {
        self.indexed_reader.fetch_all_by_rid(record_index).unwrap();

        self.clear_sequence_buffer(); // TODO: Create buffer with capacity
        self.indexed_reader.read(&mut self.sequence_buffer).unwrap();

        &self.sequence_buffer
    }
}

// TODO: Add Labeling trait for IndexedFastaProvider