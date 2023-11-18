/*!
Provides the `PatternIndex` and its basic implementations.
*/
pub trait PatternIndex: Sized {
    type Option;
    type BuildError: std::error::Error;

    /// Create a new `PatternIndex` instance with the given concatenated sequence.
    fn new(concatenated_sequence : Vec<u8>, option: Self::Option) -> Result<Self, Self::BuildError>;
    /// Get sorted positions of the given pattern in concatenated sequence.
    fn get_sorted_positions(&self, pattern: &[u8]) -> Vec<u32>;
}
