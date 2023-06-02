use super::{
    Reference,
    PatternIndex,
    SequenceStorage,
};

impl<I, S> Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    pub fn is_alignable(&self, query: &[u8]) -> bool {
        self.sequence_type.is_alignable(query)
    }
    pub fn alignable_bytes(&self) -> Vec<u8> {
        self.sequence_type.alignable_sequence()
    }
    pub fn num_targets(&self) -> u32 {
        self.sequence_storage.num_targets()
    }
}
