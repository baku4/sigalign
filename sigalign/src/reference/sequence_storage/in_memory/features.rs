use crate::reference::extensions::Serialize;
use super::{InMemoryStorage};


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