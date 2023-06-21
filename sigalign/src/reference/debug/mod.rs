use super::{
    Reference,
    PatternIndex,
    SequenceStorage,
};

impl<I, S> Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    pub fn num_targets(&self) -> u32 {
        self.sequence_storage.num_targets()
    }
}
