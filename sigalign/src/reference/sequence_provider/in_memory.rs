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
    Serializable,
    LabelProvider,
    ReverseComplement,
};

use crate::util::FastaReader;

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
        //  - Size
        writer.write_u64::<EndianType>(self.combined_sequence.len() as u64)?;
        //  - inner bytes
        writer.write_all(&self.combined_sequence)?;

        // 3. Write sequence_index
        //  - Size
        writer.write_u64::<EndianType>(self.sequence_index.len() as u64)?;
        //  - inner bytes
        #[cfg(target_pointer_width="64")]
        self.sequence_index.iter().for_each(|position| {
            writer.write_u64::<EndianType>(*position as u64);
        });
        #[cfg(target_pointer_width="32")]
        self.sequence_index.iter().for_each(|position| {
            writer.write_u32::<EndianType>(*position as u32);
        });

        // 4. Write combined_label
        let combined_label_byte = self.combined_label.as_bytes();
        //  - Size
        writer.write_u64::<EndianType>(combined_label_byte.len() as u64)?;
        //  - inner bytes
        writer.write_all(combined_label_byte)?;

        // 5. Write label_index
        //  - Size
        writer.write_u64::<EndianType>(self.label_index.len() as u64)?;
        // - inner bytes
        #[cfg(target_pointer_width="64")]
        self.label_index.iter().for_each(|position| {
            writer.write_u64::<EndianType>(*position as u64);
        });
        #[cfg(target_pointer_width="32")]
        self.label_index.iter().for_each(|position| {
            writer.write_u32::<EndianType>(*position as u32);
        });

        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self> where
        R: std::io::Read,
        Self: Sized,
    {
        // 1. Read record_count
        let record_count = reader.read_u64::<EndianType>()? as  usize;

        // 2. Read combined_sequence
        let combined_sequence_size = reader.read_u64::<EndianType>()? as  usize;
        let mut combined_sequence = vec![0; combined_sequence_size];
        reader.read_exact(&mut combined_sequence)?;

        // 3. Read sequence_index
        let sequence_index_size = reader.read_u64::<EndianType>()? as  usize;
        #[cfg(target_pointer_width="64")]
        let sequence_index: Vec<usize> = {
            let mut sequence_index = vec![0; sequence_index_size];
            reader.read_u64_into::<EndianType>(&mut sequence_index)?;
            sequence_index.into_iter().map(|x| x as usize).collect()
        };
        #[cfg(target_pointer_width="32")]
        let sequence_index: Vec<usize> = {
            let mut sequence_index = vec![0; sequence_index_size];
            reader.read_u32_into::<EndianType>(&mut sequence_index)?;
            sequence_index.into_iter().map(|x| x as usize).collect()
        };

        // 4. Read combined_label
        let combined_label_byte_size =  reader.read_u64::<EndianType>()? as usize;
        let mut combined_label_byte = vec![0; combined_label_byte_size];
        reader.read_exact(&mut combined_label_byte)?;
        let combined_label = unsafe {
            String::from_utf8_unchecked(combined_label_byte)
        };

        // 5. Read label_index
        let label_index_size = reader.read_u64::<EndianType>()? as  usize;
        #[cfg(target_pointer_width="64")]
        let label_index: Vec<usize> = {
            let mut label_index = vec![0; label_index_size];
            reader.read_u64_into::<EndianType>(&mut label_index)?;
            label_index.into_iter().map(|x| x as usize).collect()
        };
        #[cfg(target_pointer_width="32")]
        let label_index = {
            let mut label_index = vec![0; label_index_size];
            reader.read_u32_into::<EndianType>(&mut label_index)?;
            label_index.into_iter().map(|x| x as usize).collect()
        };

        Ok(Self {
            record_count,
            combined_sequence,
            sequence_index,
            combined_label,
            label_index,
        })
    }
}
