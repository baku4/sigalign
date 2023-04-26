use super::{
    Reference,
    SequenceType,
    PatternIndex,
    SequenceStorage,
};

impl<I, S> Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    pub fn validate_query(&self, query: &[u8]) -> bool {
        self.sequence_type.validate_query(query)
    }
    pub fn valid_characters(&self) -> Vec<u8> {
        self.sequence_type.valid_characters()
    }
}