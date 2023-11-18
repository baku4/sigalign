use sigalign_core::reference::{
    SequenceStorage,
    SequenceBuffer,
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
    pub fn get_label_safely(&self, target_index: u32) -> Option<String> {
        if target_index as usize >= self.target_count {
            return None
        }
        Some(self.label_of_target_unchecked(target_index))
    }
}

// Extensions
use sigalign_core::reference::extensions::{
    Serialize,
    EstimateSize,
    LabelStorage,
};
//  - Serialize
use crate::core::{EndianType, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write, Error, ErrorKind};
use capwriter::{Save, Load};
impl Serialize for InMemoryStorage {
    fn save_to<W>(&self, mut writer: W) -> Result<(), Error> where
        W: Write
    {
        writer.write_u64::<EndianType>(self.target_count as u64)?;
        self.concatenated_sequence.save_to(&mut writer)?;
        self.sequence_index.save_to(&mut writer)?;
        self.concatenated_label.as_bytes().save_to(&mut writer)?;
        self.label_index.save_to(&mut writer)?;
        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self, Error> where
        R: Read,
        Self: Sized,
    {
        let target_count = reader.read_u64::<EndianType>()? as usize;
        let concatenated_sequence = Vec::load_from(&mut reader)?;
        let sequence_index = Vec::load_from(&mut reader)?;
        let concatenated_label = match String::from_utf8(Vec::<u8>::load_from(&mut reader)?) {
            Ok(v) => v,
            Err(_) => return Err(ErrorKind::InvalidData.into()),
        };
        let label_index = Vec::load_from(&mut reader)?;
        Ok(Self {
            target_count,
            concatenated_sequence,
            sequence_index,
            concatenated_label,
            label_index,
        })
    }
}
//  - EstimateSize
impl EstimateSize for InMemoryStorage {
    fn serialized_size(&self) -> usize {
        // target_count
        std::mem::size_of::<u64>()
        // concatenated_sequence
        + self.concatenated_sequence.to_be_saved_size()
        // sequence_index
        + self.sequence_index.to_be_saved_size()
        // concatenated_label
        + self.concatenated_label.as_bytes().to_be_saved_size()
        // label_index
        + self.label_index.to_be_saved_size()
    }
}
//  - Label Storage
impl LabelStorage for InMemoryStorage {
    fn label_of_target_unchecked(&self, target_index: u32) -> String {
        unsafe {
            String::from_utf8_unchecked(
                self.concatenated_label.as_bytes()[
                    self.label_index[target_index as usize]
                    ..self.label_index[target_index as usize +1]
                ].to_vec()
            )
        }
    }
}
