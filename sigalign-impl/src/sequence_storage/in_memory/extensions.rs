use std::io::{Read, Write, Error, ErrorKind};

use capwriter::{Save, Load};

use sigalign_core::reference::extensions::{
    Serialize,
    EstimateSize,
    LabelStorage,
};
use crate::core::{EndianType, ReadBytesExt, WriteBytesExt};
use super::InMemoryStorage;

//  - Serialize
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
impl InMemoryStorage {
    pub fn get_label_safely(&self, target_index: u32) -> Option<String> {
        if target_index as usize >= self.target_count {
            return None
        }
        Some(self.label_of_target_unchecked(target_index))
    }
}
