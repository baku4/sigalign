use super::{
    Reference,
    PatternIndex,
    SequenceStorage,
};

impl<I, S> Clone for Reference<I, S> where
    I: PatternIndex + Clone,
    S: SequenceStorage + Clone,
{
    fn clone(&self) -> Self {
        Self {
            target_boundaries: self.target_boundaries.clone(),
            pattern_index: self.pattern_index.clone(),
            sequence_storage: self.sequence_storage.clone(),
        }
    }
}
