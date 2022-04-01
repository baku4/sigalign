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
    fn split_by_max_length(self, max_length: usize) -> Result<Vec<Self>> {
        // Get record index range list
        let record_index_range_list = self.record_index_range_list_of_max_length(max_length);

        Ok(Vec::new())
    }
}
impl InMemoryProvider {
    fn record_index_range_list_of_max_length(&self, max_length: usize) -> Vec<(usize, usize)> {
        let mut record_index_range_list = Vec::new(); // (start index, last index)
        let mut start_record_index = 0;
        let mut max_accumulated_length = max_length;

        for (record_index, accumulated_length) in self.sequence_index[1..].iter().enumerate() {
            if max_accumulated_length < *accumulated_length {
                if record_index == start_record_index { // One record exceed the max length
                    record_index_range_list.push((start_record_index, record_index));
                    start_record_index = record_index + 1;
                    max_accumulated_length = *accumulated_length + max_length;
                } else {
                    record_index_range_list.push((start_record_index, record_index - 1));
                    start_record_index = record_index;
                    max_accumulated_length = self.sequence_index[record_index - 1] + max_length;
                }
            }
        }

        record_index_range_list        
    }
}

#[test]
fn record_index_range_list_of_max_length() {
    let in_memory_provider = InMemoryProvider {
        record_count: 10,
        combined_sequence: Vec::new(),
        sequence_index: Vec<usize>,
        combined_label: String,
        label_index: Vec::new(),
    };
}
