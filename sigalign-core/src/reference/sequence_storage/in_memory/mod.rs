use super::{
    SequenceStorage, SequenceBuffer,
    ConcatenatedSequenceWithBoundaries,
};
use crate::reference::extensions::{
    Serialize,
    EstimateSize,
    LabelStorage,
};
use crate::utils::reverse_complement_of_dna;

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
    fn get_concatenated_sequence_with_boundaries(&self) -> ConcatenatedSequenceWithBoundaries {
        ConcatenatedSequenceWithBoundaries {
            concatenated_sequence: self.concatenated_sequence.to_vec(),
            boundaries: self.sequence_index.iter().map(|x| *x as u64).collect(),
        }
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
    pub fn to_reverse_complement(&self) -> Self {
        let mut new_combined_sequence: Vec<u8> = Vec::with_capacity(self.concatenated_sequence.len());
        (0..self.target_count).for_each(|idx| {
            let start_idx = self.sequence_index[idx];
            let end_idx = self.sequence_index[idx+1];
            let org_seq = &self.concatenated_sequence[start_idx..end_idx];
            let mut rc_seq = reverse_complement_of_dna(org_seq);
            new_combined_sequence.append(&mut rc_seq);
        });

        Self {
            target_count: self.target_count,
            concatenated_sequence: new_combined_sequence,
            sequence_index: self.sequence_index.clone(),
            concatenated_label: self.concatenated_label.clone(),
            label_index: self.label_index.clone(),
        }
    }
    pub fn append_reverse_complement(&mut self) {
        // combined_sequence
        self.concatenated_sequence.reserve(self.concatenated_sequence.len());
        (0..self.target_count).for_each(|idx| {
            let start_idx = self.sequence_index[idx];
            let end_idx = self.sequence_index[idx+1];
            let org_seq = &self.concatenated_sequence[start_idx..end_idx];
            let mut rc_seq = reverse_complement_of_dna(org_seq);
            self.concatenated_sequence.append(&mut rc_seq);
        });
        // sequence_index
        {
            self.sequence_index.reserve(self.target_count);
            let last_seq_idx = *self.sequence_index.last().unwrap();
            for idx in 1..=self.target_count {
                let v = self.sequence_index[idx];
                self.sequence_index.push(v+last_seq_idx);
            };
        }
        // combined_label
        self.concatenated_label.push_str(&self.concatenated_label.clone());
        // label_index
        {
            self.label_index.reserve(self.target_count);
            let last_label_idx = *self.label_index.last().unwrap();
            for idx in 1..=self.target_count {
                let v = self.label_index[idx];
                self.label_index.push(v+last_label_idx);
            };
        }
        // record_count
        self.target_count <<= 1;
    }
    pub fn merge(&mut self, other: Self) {
        let Self {
            target_count: other_record_count,
            concatenated_sequence: mut other_combined_sequence,
            sequence_index: other_sequence_index,
            concatenated_label: other_combined_label,
            label_index: other_label_index,
        } = other;
        // record_count
        self.target_count += other_record_count;
        // concatenated_sequence
        self.concatenated_sequence.append(&mut other_combined_sequence);
        // sequence_index
        let last_seq_idx = *self.sequence_index.last().unwrap();
        self.sequence_index.reserve(other_record_count);
        other_sequence_index[1..].iter().for_each(|v| {
            self.sequence_index.push(v+last_seq_idx);
        });
        // concatenated_label
        self.concatenated_label.push_str(&other_combined_label);
        // label_index
        let last_label_idx = *self.label_index.last().unwrap();
        self.label_index.reserve(other_record_count);
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
    pub fn get_label_safely(&self, target_index: u32) -> Option<String> {
        if target_index as usize >= self.target_count {
            return None
        }
        Some(self.label_of_target_unchecked(target_index))
    }
}

// Features
mod io;
mod label;
