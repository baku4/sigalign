use super::{
    Reference,
    PatternIndex,
    SequenceStorage,
};
use std::io::{Write, Read, Error};

/// Save and load the structure
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
        self.target_boundaries.save_to(&mut writer)?;
        self.pattern_index.save_to(&mut writer)?;
        self.sequence_storage.save_to(&mut writer)?;
        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self, Error> where
        R: Read,
        Self: Sized
    {
        let target_boundaries = Vec::load_from(&mut reader)?;
        let pattern_index = I::load_from(&mut reader)?;
        let sequence_storage = S::load_from(&mut reader)?;
        Ok(Self {
            target_boundaries,
            pattern_index,
            sequence_storage,
        })
    }
}

/// Provides an estimate of the size of the object when saved.
pub trait EstimateSize {
    fn serialized_size(&self) -> usize;
}

impl<I, S> EstimateSize for Reference<I, S> where
    I: PatternIndex + EstimateSize,
    S: SequenceStorage + EstimateSize,
{
    fn serialized_size(&self) -> usize {
        (self.target_boundaries.len() * std::mem::size_of::<u32>())
        + self.sequence_storage.serialized_size()
        + self.pattern_index.serialized_size()
    }
}
