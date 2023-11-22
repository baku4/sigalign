use std::{io::Read, str::Utf8Error};

use sigalign_core::reference::{
    SequenceStorage,
    SequenceBuffer,
};
use sigalign_utils::sequence_reader::{
    SeqRecord, IdRecord,
    fasta::FastaReader,
    decompress::{get_gzip_decoder, get_zlib_decoder},
};

// TODO: Debug impl manually
/// Basic `SequenceStorage` implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InMemoryStorage {
    target_count: usize,
    concatenated_sequence: Vec<u8>,
    sequence_index: Vec<usize>,
    concatenated_label: String,
    label_index: Vec<usize>,
}
pub struct InMemoryBuffer {
    pointer: *const u8,
    len: usize,
}

// Sequence Storage
impl SequenceStorage for InMemoryStorage {
    type Buffer = InMemoryBuffer;

    fn num_targets(&self) -> u32 {
        self.target_count as u32
    }
    fn get_buffer(&self) -> Self::Buffer {
        InMemoryBuffer {
            pointer: self.concatenated_sequence.as_ptr(),
            len: 0,
        }
    }
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer) {
        let start_index = self.sequence_index[target_index as usize];
        buffer.pointer = &self.concatenated_sequence[start_index];
        buffer.len = self.sequence_index[target_index as usize +1] - start_index;
    }
    fn get_concatenated_sequence_with_boundaries_of_targets(&self) -> (
        Vec<u8>,
        Vec<u32>,
    ) {
        let concatenated_sequence = self.concatenated_sequence.to_vec();
        let boundaries = self.sequence_index.iter().map(|x| *x as u32).collect();
        (concatenated_sequence, boundaries)
    }
}
impl SequenceBuffer for InMemoryBuffer {
    fn buffered_sequence(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.pointer, self.len) }
    }
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            target_count: 0,
            concatenated_sequence: Vec::new(),
            sequence_index: vec![0],
            concatenated_label: String::new(),
            label_index: vec![0],
        }
    }
    pub fn add_target(
        &mut self,
        label: &str,
        sequence: &[u8],
    ) {
        self.target_count += 1;
        self.concatenated_sequence.extend_from_slice(sequence);
        self.sequence_index.push(self.concatenated_sequence.len());
        self.concatenated_label.push_str(label);
        self.label_index.push(self.concatenated_label.len());
    }
    pub fn add_fasta<R: Read>(&mut self, reader: R) -> Result<(), Utf8Error> {
        let mut fasta_reader = FastaReader::new(reader);
        while let Some(mut record) = fasta_reader.next() {
            self.target_count += 1;
            record.extend_seq_buf(&mut self.concatenated_sequence);
            self.sequence_index.push(self.concatenated_sequence.len());
            record.extend_id_string(&mut self.concatenated_label)?;
            self.label_index.push(self.concatenated_label.len());
        }
        Ok(())
    }
    pub fn add_gzip_fasta<R: Read>(&mut self, reader: R) -> Result<(), Utf8Error> {
        let decomp_reader = get_gzip_decoder(reader);
        let mut fasta_reader = FastaReader::new(decomp_reader);
        while let Some(mut record) = fasta_reader.next() {
            self.target_count += 1;
            record.extend_seq_buf(&mut self.concatenated_sequence);
            self.sequence_index.push(self.concatenated_sequence.len());
            record.extend_id_string(&mut self.concatenated_label)?;
            self.label_index.push(self.concatenated_label.len());
        }
        Ok(())
    }
    pub fn merge(&mut self, other: Self) {
        let Self {
            target_count: other_target_count,
            concatenated_sequence: mut other_combined_sequence,
            sequence_index: other_sequence_index,
            concatenated_label: other_combined_label,
            label_index: other_label_index,
        } = other;
        // record_count
        self.target_count += other_target_count;
        // concatenated_sequence
        self.concatenated_sequence.append(&mut other_combined_sequence);
        // sequence_index
        let last_seq_idx = *self.sequence_index.last().unwrap();
        self.sequence_index.reserve(other_target_count);
        other_sequence_index[1..].iter().for_each(|v| {
            self.sequence_index.push(v+last_seq_idx);
        });
        // concatenated_label
        self.concatenated_label.push_str(&other_combined_label);
        // label_index
        let last_label_idx = *self.label_index.last().unwrap();
        self.label_index.reserve(other_target_count);
        other_label_index[1..].iter().for_each(|v| {
            self.label_index.push(v+last_label_idx);
        });
    }
    pub fn get_sequence_safely(&self, target_index: u32) -> Option<Vec<u8>> {
        if target_index as usize >= self.target_count {
            return None
        }
        let mut buffer = self.get_buffer();
        self.fill_buffer(target_index, &mut buffer);
        let seq = buffer.buffered_sequence().to_vec();
        Some(seq)
    }
    pub fn get_total_length(&self) -> u32 {
        self.concatenated_sequence.len() as u32
    }
}

mod extensions;
