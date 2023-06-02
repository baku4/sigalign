use crate::core::BufferedPatternSearch;
use super::Reference;
use super::pattern_index::{PatternIndex, PatternLocation};
use super::sequence_storage::SequenceStorage;

impl<I, S> Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    #[inline]
    pub fn locate_pattern(&self, pattern: &[u8]) -> Vec<PatternLocation> {
        self.locate(pattern)
    }
    #[inline]
    pub fn get_sequence_buffer(&self) -> S::Buffer {
        self.sequence_storage.get_buffer()
    }
    #[inline]
    pub fn fill_sequence_buffer(&self, target_index: u32, buffer: &mut S::Buffer) {
        self.sequence_storage.fill_buffer(target_index, buffer)
    }
}

impl<I, S> BufferedPatternSearch for Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    type Buffer = S::Buffer;

    fn locate(&self, pattern: &[u8]) -> Vec<PatternLocation> {
        self.pattern_index.locate(pattern, &self.search_range)
    }
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer) {
        self.sequence_storage.fill_buffer(target_index, buffer)
    }
}
