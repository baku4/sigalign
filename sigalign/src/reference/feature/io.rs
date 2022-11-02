use super::{
    Result,
};
use super::{
    Reference, SequenceStorage,
    // Traits implemented by structures
    Serializable, SizeAware,
    // Common data structures for Reference
    SequenceType, PatternFinder,
};

use std::io::{Write, Read};
use std::path::Path;
use std::fs::File;

use capwriter::{Saveable, Loadable};

impl<S> Reference<S> where
    S: SequenceStorage + Serializable,
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
        // Save 'sequence_storage'
        self.sequence_storage.save_to(&mut writer)?;
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
        // Load 'sequence_storage'
        let sequence_storage = S::load_from(&mut reader)?;
        Ok(Self {
            sequence_type,
            pattern_finder,
            target_record_index,
            sequence_storage: sequence_storage,
        })
    }
}

impl<S> Reference<S> where
    S: SequenceStorage + Serializable + SizeAware,
{
    /// Save to file
    pub fn save_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let file = File::create(file_path)?;
        let file_size = self.size_of();
        file.set_len(file_size as u64)?;

        self.save_to(file)?;
        Ok(())
    }
    /// Precalculate size of saved file
    pub fn size_of(&self) -> usize {
        self.target_record_index.size_of() // target_record_index
        + self.sequence_type.size_of() // sequence_type
        + self.pattern_finder.size_of() // pattern_finder
        + self.sequence_storage.size_of() // sequence_storage
    }
}
