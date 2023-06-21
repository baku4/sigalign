use super::{
    Reference,
    PatternIndex,
    SequenceStorage,
};

impl<I, S> Reference<I, S> where
    I: PatternIndex ,
    S: SequenceStorage + LabelStorage,
{
    pub fn label_of_target(&self, target_index: u32) -> Option<String> {
        if target_index < self.num_targets() {
            Some(self.label_of_target_unchecked(target_index))
        } else {
            None
        }
    }
    pub fn label_of_target_unchecked(&self, target_index: u32) -> String {
        self.sequence_storage.label_of_target_unchecked(target_index)
    }
}

/// Storage for label of sequences.
pub trait LabelStorage {
    fn label_of_target_unchecked(&self, target_index: u32) -> String;
}
