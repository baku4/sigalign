use super::{
    Reference,
    PatternIndex,
    SequenceStorage,
};

use thiserror::Error;
/// Defines the potential errors when attempting to set the search range.
#[derive(Debug, Error)]
pub enum SetSearchRangeError {
    #[error("Target index cannot be empty")]
    EmptyIndexError,
    #[error("Index cannot be over the total target")]
    IndexOutOfRangeError,
}

impl<I, S> Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    pub fn get_search_range(&self) -> &[u32] {
        &self.search_range
    }
    pub fn set_search_range(&mut self, mut target_index: Vec<u32>) -> Result<(), SetSearchRangeError> {
        target_index.sort_unstable();
        let last_record_index = match target_index.last() {
            Some(v) => v,
            None => return Err(SetSearchRangeError::EmptyIndexError),
        };
        let total_target_count = self.sequence_storage.num_targets() as u32;
        if total_target_count < *last_record_index {
            return Err(SetSearchRangeError::IndexOutOfRangeError);
        } else {
            self.set_search_range_unchecked(target_index);
            Ok(())
        }
    }
    fn set_search_range_unchecked(&mut self, sorted_target_index: Vec<u32>) {
        self.search_range = sorted_target_index;
    }
}
