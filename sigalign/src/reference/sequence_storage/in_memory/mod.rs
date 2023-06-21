use super::{
    SequenceStorage, SequenceBuffer,
    ConcatenatedSequenceWithBoundaries,
};
use crate::reference::extensions::{
    Serialize,
    EstimateSize,
    LabelStorage,
};
use crate::utils::{FastaReader, reverse_complement_of_dna};

// mod reverse_complement;
// pub use reverse_complement::InMemoryRcStorage;


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
    pub fn add_fasta_file<P>(&mut self, file_path: P) -> Result<(), std::io::Error> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_path(file_path)?;
        self.add_from_fasta_reader(fasta_reader);
        Ok(())
    }
    pub fn add_fasta_bytes(&mut self, fasta_bytes: &[u8]) {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        self.add_from_fasta_reader(fasta_reader);
    }
    fn add_from_fasta_reader<R>(&mut self, fasta_reader: FastaReader<R>) where
        R: std::io::Read,
    {
        fasta_reader.for_each(|(label, sequence)| {
            self.add_target(&label, &sequence);
        });
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


// // Divisible
// impl Divide for InMemoryStorage {
//     fn divide_into(self, max_seq_len: usize) -> Result<Vec<Self>> {
//         // Get record index range list
//         let record_index_range_list = self.record_index_range_list_of_max_length(max_seq_len);

//         // Split
//         let splitted = self.split_using_record_index_range_list(record_index_range_list);

//         Ok(splitted)
//     }
// }
// impl InMemoryStorage {
//     fn record_index_range_list_of_max_length(&self, max_seq_len: usize) -> Vec<(usize, usize)> {
//         let mut record_index_range_list = Vec::new(); // (start index, last index)
//         let mut start_record_index = 0;

//         'outer: loop {
//             let mut first_max_over_record_index = start_record_index + 1;

//             while self.sequence_index[first_max_over_record_index] - self.sequence_index[start_record_index] <= max_seq_len {
//                 first_max_over_record_index += 1;
//                 if first_max_over_record_index == self.sequence_index.len() - 1 {
//                     record_index_range_list.push((start_record_index, first_max_over_record_index));
//                     break 'outer;
//                 }
//             }

//             // accumulated_length > max_length
//             if first_max_over_record_index == start_record_index + 1 { // One record exceed the max length
//                 record_index_range_list.push((start_record_index, first_max_over_record_index));
//                 start_record_index = first_max_over_record_index;
//             } else {
//                 record_index_range_list.push((start_record_index, first_max_over_record_index - 1));
//                 start_record_index = first_max_over_record_index - 1;
//             }
//         }
//         record_index_range_list
//     }
//     fn split_using_record_index_range_list(self, record_index_range_list: Vec<(usize, usize)>) -> Vec<Self> {
//         record_index_range_list.into_iter().map(|(start_record_index, last_record_index)| {
//             // record_count
//             let record_count = last_record_index - start_record_index;

//             // combined_sequence
//             let sequence_start_index = self.sequence_index[start_record_index];
//             let sequence_end_index = self.sequence_index[last_record_index];
            
//             let combined_sequence = self.concatenated_sequence[sequence_start_index..sequence_end_index].to_vec();

//             // sequence_index
//             let sequence_index: Vec<usize> = self.sequence_index[start_record_index..=last_record_index].iter().map(|v| {
//                 v - sequence_start_index
//             }).collect();

//             // combined_label
//             let label_start_index = self.label_index[start_record_index];
//             let label_end_index = self.label_index[last_record_index];

//             let combined_label = self.combined_label[label_start_index..label_end_index].to_string();

//             // label_index
//             let label_index: Vec<usize> = self.label_index[start_record_index..=last_record_index].iter().map(|v| {
//                 v - label_start_index
//             }).collect();

//             Self {
//                 target_count: record_count,
//                 concatenated_sequence: combined_sequence,
//                 sequence_index,
//                 combined_label,
//                 label_index,
//             }
//         }).collect()
//     }
// }
