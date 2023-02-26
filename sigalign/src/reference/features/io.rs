use super::{
    Reference,
    SequenceType,
    PatternIndex,
    SequenceStorage,
};

/// Save and load the structure
use std::io::{Write, Read, Error};
pub trait Serialize {
    fn save_to<W>(&self, writer: W) -> Result<(), Error> where
        W: Write;
    fn load_from<R>(reader: R) -> Result<Self, Error> where
        R: Read,
        Self: Sized;
}

use capwriter::{Save, Load};
impl<I, S> Serialize for Reference<I, S> where
    I: PatternIndex + Serialize,
    S: SequenceStorage + Serialize,
{
    fn save_to<W>(&self, mut writer: W) -> Result<(), Error> where
        W: Write
    {
        self.sequence_type.save_to(&mut writer)?;
        self.search_range.save_to(&mut writer)?;
        self.pattern_index.save_to(&mut writer)?;
        self.sequence_storage.save_to(&mut writer)?;
        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self, Error> where
        R: Read,
        Self: Sized
    {
        let sequence_type = SequenceType::load_from(&mut reader)?;
        let search_range = Vec::load_from(&mut reader)?;
        let pattern_index = I::load_from(&mut reader)?;
        let sequence_storage = S::load_from(&mut reader)?;
        Ok(Self {
            sequence_type,
            search_range,
            pattern_index,
            sequence_storage,
        })
    }
}
