use thiserror::Error;

use crate::utils::get_unique_characters_of_sequence;
use sigalign_core::reference::PatternIndex;
use lt_fm_index::{
    LtFmIndex, Block, blocks,
};

/// `StaticLfi` that can index 3 (2^2 - 1) characters, with a BWT block size of 64.
pub type Lfi32B2V64 = StaticLfi<blocks::Block2<u64>>;
/// `StaticLfi` that can index 7 (2^3 - 1) characters, with a BWT block size of 64.
pub type Lfi32B3V64 = StaticLfi<blocks::Block3<u64>>;
/// `StaticLfi` that can index 15 (2^4 - 1) characters, with a BWT block size of 64.
pub type Lfi32B4V64 = StaticLfi<blocks::Block4<u64>>;
/// `StaticLfi` that can index 31 (2^5 - 1) characters, with a BWT block size of 64.
pub type Lfi32B5V64 = StaticLfi<blocks::Block5<u64>>;

// TODO: Check if the specification is accurate.
/// LtFmIndex that has a maximum number of characters that can be indexed.
/// (The maximum length of one sequence is u32::MAX)
#[derive(Clone)]
pub struct StaticLfi<B: Block<u32>> {
    inner: LtFmIndex<u32, B>,
}

#[derive(Debug, Clone)]
/// Option to define the structure of the LtFmIndex.
pub struct LfiOption {
    pub suffix_array_sampling_ratio: u64,
    pub lookup_table_max_bytes_size : u64,
    pub use_safe_guard: bool,
}
impl LfiOption {
    pub fn new(
        suffix_array_sampling_ratio: u64,
        lookup_table_max_bytes_size: u64,
        use_safe_guard: bool,
    ) -> Self {
        Self {
            suffix_array_sampling_ratio,
            lookup_table_max_bytes_size,
            use_safe_guard,
        }
    }
}

impl <B: Block<u32>> PatternIndex for StaticLfi<B> {
    type Option = LfiOption;
    type BuildError = LfiBuildError;
    
    fn new(concatenated_sequence : Vec<u8>, option: Self::Option) -> Result<Self, Self::BuildError> {
        let unique_sequence = get_unique_characters_of_sequence(&concatenated_sequence);
        let mut valid_characters: Vec<Vec<u8>> = unique_sequence.into_iter().map(|v| vec![v]).collect();
        if !option.use_safe_guard {
            valid_characters.pop(); // Remove last character
        }
        let characters_by_index: Vec<&[u8]> = valid_characters.iter()
            .map(|v| v.as_slice())
            .collect();
        if characters_by_index.len() as u32 > B::MAX_CHR {
            let err: LfiBuildError = Self::BuildError::OverMaximumCharacters {
                max: B::MAX_CHR,
                input: characters_by_index.len() as u32,
            };
            return Err(err);
        }

        let sequence_length = concatenated_sequence.len();
        if sequence_length >= u32::MAX as usize {
            return Err(Self::BuildError::SequenceLengthOver(u32::MAX as u64));
        }
        let lookup_table_kmer_size = calculate_lookup_table_kmer_size(
            characters_by_index.len(),
            option.lookup_table_max_bytes_size as usize,
        );

        match LtFmIndex::build(
            concatenated_sequence,
            &characters_by_index,
            option.suffix_array_sampling_ratio as u32,
            lookup_table_kmer_size,
        ) {
            Ok(v) => Ok(Self { inner: v }),
            Err(err) => Err(Self::BuildError::InvalidOption(format!("{}", err))),
        }
    }
    fn get_sorted_positions(&self, pattern: &[u8]) -> Vec<u32> {
        let mut positions = self.inner.locate(pattern);
        positions.sort_unstable();
        positions
    }
}

fn calculate_lookup_table_kmer_size(
    chr_count: usize,
    maximum_bytes_size: usize,
) -> u32 {
    let max_cap = 50;
    for v in 1..=max_cap {
        let estimated_byte_size_of_lt = (chr_count+1).pow(v);
        if estimated_byte_size_of_lt >= maximum_bytes_size {
            return v - 1
        }
    }
    max_cap
}

/// Error type for `StaticLfi` build.
#[derive(Debug, Error)]
pub enum LfiBuildError {
    /// Triggered when sequence length exceeds the maximum allowable capacity.
    #[error("Sequence length is over the maximum capacity {0}")]
    SequenceLengthOver(u64),
    /// Triggered when input characters exceed the maximum limit that the `PatternIndex` can index.
    #[error("Pattern index can make index of {max} characters, input is {input}")]
    OverMaximumCharacters{
        max: u32,    // The maximum number of characters that PatternIndex can index
        input: u32,  // Input characters
    },
    /// Triggered when the invalid option is passed.
    #[error("Error in option: {0}")]
    InvalidOption(String), // Error message
}

// Impl Extensions
use sigalign_core::reference::extensions::{
    Serialize,
    EstimateSize,
};
//  - Serialize
impl<B: Block<u32>> Serialize for StaticLfi<B> {
    fn save_to<W>(&self, mut writer: W) -> Result<(), std::io::Error> where
        W: std::io::Write
    {
        self.inner.save_to(&mut writer)?;
        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self, std::io::Error> where
        R: std::io::Read,
        Self: Sized
    {
        let inner = LtFmIndex::load_from(&mut reader)?;
        Ok(Self { inner })
    }
}
//  - EstimateSize
impl<B: Block<u32>> EstimateSize for StaticLfi<B> {
    fn serialized_size(&self) -> usize {
        self.inner.to_be_saved_size()
    }
}
