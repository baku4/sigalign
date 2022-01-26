use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder,
};

use std::io::{Write, Read};
use crate::{EndianType, SizedUint};
use byteorder::{ReadBytesExt, WriteBytesExt};

pub trait Serializable {
    fn save_to<W>(&self, writer: W) -> Result<()> where W: Write;
    fn load_from<R>(reader: R) -> Result<Self> where R: Read, Self: Sized;
}

impl<S> Reference<S> where
    S: SequenceProvider + Serializable,
{
    pub fn save_to<W>(&self, mut writer: W) -> Result<()> where
        W: Write,
    {
        // Save 'target_record_index'
        let target_record_index_size = self.target_record_index.len() as u32;
        writer.write_u32::<EndianType>(target_record_index_size)?;
        self.target_record_index.iter().for_each(|record_index| {
            writer.write_u32::<EndianType>(*record_index);
        });

        // Save 'sequence_type'
        self.sequence_type.save_to(&mut writer)?;
        // Save 'pattern_finder'
        self.pattern_finder.save_to(&mut writer)?;
        // Save 'sequence_provider'
        self.sequence_provider.save_to(&mut writer)?;
        Ok(())
    }
    pub fn load_from<R>(mut reader: R) -> Result<Self> where
        R: Read,
        Self: Sized,
    {
        // Load 'target_record_index'
        let target_record_index_size = reader.read_u32::<EndianType>()? as usize;
        let mut target_record_index = vec![0; target_record_index_size];
        reader.read_u32_into::<EndianType>(&mut target_record_index)?;

        // Load 'sequence_type'
        let sequence_type = SequenceType::load_from(&mut reader)?;
        // Load 'pattern_finder'
        let pattern_finder = PatternFinder::load_from(&mut reader)?;
        // Load 'sequence_provider'
        let sequence_provider = S::load_from(&mut reader)?;
        Ok(Self {
            sequence_type,
            pattern_finder,
            target_record_index,
            sequence_provider,
        })
    }
}
