use super::{
    Reference,
    PatternIndex,
    SequenceStorage,
};

impl<I, S> Reference<I, S> where
    I: PatternIndex ,
    S: SequenceStorage + LabelStorage,
{
    pub fn label_of_target(&self, target_index: usize) -> String {
        self.sequence_storage.label_of_target(target_index)
    }
}

/// Storage for label of sequences.
pub trait LabelStorage {
    fn label_of_target(&self, target_index: usize) -> String;
}
