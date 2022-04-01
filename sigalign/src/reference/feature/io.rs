use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceProvider,
    // Traits implemented by structures
    Serializable, SizeAware,
    // Common data structures for Reference
    SequenceType, PatternFinder,
};

use std::io::{Write, Read};
use std::path::Path;
use std::fs::File;

use crate::{EndianType, SizedUint};
use byteorder::{ReadBytesExt, WriteBytesExt};
use capwriter::{Saveable, Loadable};

impl<S> Reference<S> where
    S: SequenceProvider + Serializable,
{
    pub fn save_to<W>(&self, mut writer: W) -> Result<()> where
        W: Write,
    {
        // Save 'target_record_index'
        self.target_record_index.save_to(&mut writer)?;
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
        let target_record_index = Vec::load_from(&mut reader)?;
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

impl<S> Reference<S> where
    S: SequenceProvider + Serializable + SizeAware,
{
    // Save to file
    pub fn save_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let file = File::create(file_path)?;
        let file_size = self.size_of();
        file.set_len(file_size as u64)?;

        self.save_to(file)?;
        Ok(())
    }
    fn size_of(&self) -> usize {
        self.target_record_index.size_of() // target_record_index
        + self.sequence_type.size_of() // sequence_type
        + self.pattern_finder.size_of() // pattern_finder
        + self.sequence_provider.size_of() // sequence_provider
    }
}
