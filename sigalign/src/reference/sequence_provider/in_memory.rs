use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference, SequenceProvider, JoinedSequence,
    SequenceType, PatternFinder,
    // traits
    Divisible, Serializable, SizeAware,
    LabelProvider,
    ReverseComplement,
};

use crate::util::FastaReader;

use capwriter::{Saveable, Loadable};
use serde::{Serialize, Deserialize};
use bincode::{serialize_into, deserialize_from};

mod reverse_complement;
pub use reverse_complement::InMemoryRcProvider;

/// Basic `SequenceProvider` implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InMemoryProvider {
    record_count: usize,
    combined_sequence: Vec<u8>,
    sequence_index: Vec<usize>,
    combined_label: String,
    label_index: Vec<usize>,
}

impl InMemoryProvider {
    pub fn new() -> Self {
        Self {
            record_count: 0,
            combined_sequence: Vec::new(),
            sequence_index: vec![0],
            combined_label: String::new(),
            label_index: vec![0],
        }
    }
    pub fn add_record(
        &mut self,
        sequence: &[u8],
        label: &str,
    ) {
        self.record_count += 1;
        self.combined_sequence.extend_from_slice(sequence);
        self.sequence_index.push(self.combined_sequence.len());
        self.combined_label.push_str(label);
        self.label_index.push(self.combined_label.len());
    }
    pub fn add_fasta_file<P>(&mut self, file_path: P) -> Result<()> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_file_path(file_path)?;
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
            self.add_record(&sequence, &label);
        });
    }
}

pub struct InMemoryBuffer {
    pointer: *const u8,
    len: usize,
}

impl SequenceBuffer for InMemoryBuffer {
    fn request_sequence(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.pointer, self.len) }
    }
}

// Sequence Provider
impl SequenceProvider for InMemoryProvider {
    type Buffer = InMemoryBuffer;

    fn total_record_count(&self) -> usize {
        self.record_count
    }
    fn get_buffer(&self) -> Self::Buffer {
        InMemoryBuffer {
            pointer: self.combined_sequence.as_ptr(),
            len: 0,
        }
    }
    fn fill_sequence_buffer(&self, record_index: usize, buffer: &mut Self::Buffer) {
        let start_index = self.sequence_index[record_index];
        buffer.pointer = &self.combined_sequence[start_index];
        buffer.len = self.sequence_index[record_index+1] - start_index;
    }
    fn get_joined_sequence(&self) -> JoinedSequence {
        JoinedSequence::new(
            self.combined_sequence.to_vec(),
            self.sequence_index.iter().map(|x| *x as u64).collect(),
        )
    }
}

// Label Provider
impl LabelProvider for InMemoryProvider {
    fn label_of_record(&self, record_index: usize) -> String {
        String::from(&self.combined_label[
            self.label_index[record_index]..self.label_index[record_index+1]
        ])
    }
}

use crate::{EndianType};
use byteorder::{ReadBytesExt, WriteBytesExt};

// Serializable
impl Serializable for InMemoryProvider {
    fn save_to<W>(&self, mut writer: W) -> Result<()> where
        W: std::io::Write
    {
        // 1. Write record_count
        writer.write_u64::<EndianType>(self.record_count as u64)?;
        // 2. Write combined_sequence
        self.combined_sequence.save_to(&mut writer)?;
        // 3. Write sequence_index
        self.sequence_index.save_to(&mut writer)?;
        // 4. Write combined_label
        let combined_label_byte = self.combined_label.as_bytes();
        combined_label_byte.save_to(&mut writer)?;
        // 5. Write label_index
        self.label_index.save_to(&mut writer)?;

        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self> where
        R: std::io::Read,
        Self: Sized,
    {
        // 1. Read record_count
        let record_count = reader.read_u64::<EndianType>()? as usize;
        // 2. Read combined_sequence
        let combined_sequence = Vec::load_from(&mut reader)?;
        // 3. Read sequence_index
        let sequence_index = Vec::load_from(&mut reader)?;
        // 4. Read combined_label
        let combined_label_byte = Vec::<u8>::load_from(&mut reader)?;
        let combined_label = unsafe {
            String::from_utf8_unchecked(combined_label_byte)
        };
        // 5. Read label_index
        let label_index = Vec::load_from(&mut reader)?;

        Ok(Self {
            record_count,
            combined_sequence,
            sequence_index,
            combined_label,
            label_index,
        })
    }
}

// SizeAware
impl SizeAware for InMemoryProvider {
    fn size_of(&self) -> usize {
        8 // record_count
        + self.combined_sequence.size_of() // combined_sequence
        + self.sequence_index.size_of() // sequence_index
        + self.combined_label.as_bytes().size_of() // combined_label
        + self.label_index.size_of() // label_index
    }
}

// Divisible
impl Divisible for InMemoryProvider {
    fn split_by_max_length(self, max_seq_len: usize) -> Result<Vec<Self>> {
        // Get record index range list
        let record_index_range_list = self.record_index_range_list_of_max_length(max_seq_len);

        // Split
        let splitted = self.split_using_record_index_range_list(record_index_range_list);

        Ok(splitted)
    }
}
impl InMemoryProvider {
    fn record_index_range_list_of_max_length(&self, max_seq_len: usize) -> Vec<(usize, usize)> {
        let mut record_index_range_list = Vec::new(); // (start index, last index)
        let mut start_record_index = 0;

        'outer: loop {
            let mut first_max_over_record_index = start_record_index + 1;

            while self.sequence_index[first_max_over_record_index] - self.sequence_index[start_record_index] <= max_seq_len {
                first_max_over_record_index += 1;
                if first_max_over_record_index == self.sequence_index.len() - 1 {
                    record_index_range_list.push((start_record_index, first_max_over_record_index));
                    break 'outer;
                }
            }

            // accumulated_length > max_length
            if first_max_over_record_index == start_record_index + 1 { // One record exceed the max length
                record_index_range_list.push((start_record_index, first_max_over_record_index));
                start_record_index = first_max_over_record_index;
            } else {
                record_index_range_list.push((start_record_index, first_max_over_record_index - 1));
                start_record_index = first_max_over_record_index - 1;
            }
        }
        record_index_range_list
    }
    fn split_using_record_index_range_list(self, record_index_range_list: Vec<(usize, usize)>) -> Vec<Self> {
        record_index_range_list.into_iter().map(|(start_record_index, last_record_index)| {
            // record_count
            let record_count = last_record_index - start_record_index;

            // combined_sequence
            let sequence_start_index = self.sequence_index[start_record_index];
            let sequence_end_index = self.sequence_index[last_record_index];
            
            let combined_sequence = self.combined_sequence[sequence_start_index..sequence_end_index].to_vec();

            // sequence_index
            let sequence_index: Vec<usize> = self.sequence_index[start_record_index..=last_record_index].iter().map(|v| {
                v - sequence_start_index
            }).collect();

            // combined_label
            let label_start_index = self.label_index[start_record_index];
            let label_end_index = self.label_index[last_record_index];

            let combined_label = self.combined_label[label_start_index..label_end_index].to_string();

            // label_index
            let label_index: Vec<usize> = self.label_index[start_record_index..=last_record_index].iter().map(|v| {
                v - label_start_index
            }).collect();

            Self {
                record_count,
                combined_sequence,
                sequence_index,
                combined_label,
                label_index,
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_index_range_list_of_max_length() {
        check_splitted_provider_with_fasta_file("c:\\Users\\khun\\Downloads\\sa_test_mg\\test_ref_seq.fa")
    }

    fn check_splitted_provider_with_fasta_file(fasta_file: &str) {
        // Original
        let mut in_memory_provider = InMemoryRcProvider::new();
        in_memory_provider.add_fasta_file(fasta_file);
        let mut org_label_list: Vec<String> = Vec::with_capacity(in_memory_provider.total_record_count());
        let mut org_seq_list: Vec<Vec<u8>> = Vec::with_capacity(in_memory_provider.total_record_count());
        for idx in 0..in_memory_provider.total_record_count() {
            // Label
            let label = in_memory_provider.label_of_record(idx);
            org_label_list.push(label);
            // Seq
            let mut buffer = in_memory_provider.get_buffer();
            in_memory_provider.fill_sequence_buffer(idx, &mut buffer);
            let seq = buffer.request_sequence().to_vec();
            org_seq_list.push(seq);
        }
    
        // Splitted
        let mut splitted_label_list: Vec<String> = Vec::with_capacity(in_memory_provider.total_record_count());
        let mut splitted_seq_list: Vec<Vec<u8>> = Vec::with_capacity(in_memory_provider.total_record_count());
    
        let splitted = in_memory_provider.split_by_max_length(10000).unwrap();
        println!("splitted_len: {}", splitted.len());
    
        for (idx, in_memory_provider) in splitted.into_iter().enumerate() {
            for ridx in 0..in_memory_provider.total_record_count() {
                let label = in_memory_provider.label_of_record(ridx);
                splitted_label_list.push(label);
    
                let mut buffer = in_memory_provider.get_buffer();
                in_memory_provider.fill_sequence_buffer(ridx, &mut buffer);
                // let seq = String::from_utf8(buffer.request_sequence().to_vec()).unwrap();
                let seq = buffer.request_sequence().to_vec();
                splitted_seq_list.push(seq);
            }
        }
    
        // Compare
        for idx in 0..org_label_list.len() {
            assert_eq!(org_label_list[idx], splitted_label_list[idx]);
            assert_eq!(org_seq_list[idx], splitted_seq_list[idx])
        }
    }
}
