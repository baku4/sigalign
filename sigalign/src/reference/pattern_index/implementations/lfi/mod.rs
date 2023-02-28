use super::{
    SequenceType,
    PatternIndex, PatternLocation, ConcatenatedSequenceWithBoundaries,
    PatternIndexBuildError,
    utils::{
        sorted_positions_to_pattern_location
    },
};
use lt_fm_index::{
    LtFmIndex, Block, blocks,
};

pub type Lfi32B2V64 = Lfi32<blocks::Block2<u64>>;
pub type Lfi32B3V64 = Lfi32<blocks::Block3<u64>>;
pub type Lfi32B4V64 = Lfi32<blocks::Block4<u64>>;
pub type Lfi32B5V64 = Lfi32<blocks::Block5<u64>>;

pub struct Lfi32<B: Block<u32>> {
    inner: LtFmIndex<u32, B>,
    boundaries: Vec<u32>,
}
pub struct Lfi64<B: Block<u64>> {
    inner: LtFmIndex<u64, B>,
    boundaries: Vec<u64>,
}
pub struct LfiOption {
    pub suffix_array_sampling_ratio: u64,
    pub lookup_table_kmer_size: u32,
}

impl<B: Block<u32>> PatternIndex for Lfi32<B> {
    type Option = LfiOption;

    fn new(
        concatenated_sequence_with_boundaries: ConcatenatedSequenceWithBoundaries,
        sequence_type: &SequenceType,
        option: Self::Option,
    ) -> Result<Self, PatternIndexBuildError> {
        let mut valid_characters: Vec<Vec<u8>> = sequence_type.valid_characters()
            .into_iter().map(|v| vec![v]).collect();
        valid_characters.pop(); // Remove last character
        let characters_by_index: Vec<&[u8]> = valid_characters.iter()
            .map(|v| v.as_slice())
            .collect();
        let sequence_length = concatenated_sequence_with_boundaries.concatenated_sequence.len();
        if sequence_length >= u32::MAX as usize {
            return Err(PatternIndexBuildError::SequenceLengthOver(u32::MAX as u64))
        }
        match LtFmIndex::build(
            concatenated_sequence_with_boundaries.concatenated_sequence,
            &characters_by_index,
            option.suffix_array_sampling_ratio as u32,
            option.lookup_table_kmer_size,
        ) {
            Ok(v) => {
                Ok(Self {
                    inner: v,
                    boundaries: concatenated_sequence_with_boundaries.boundaries
                        .into_iter()
                        .map(|x| x as u32)
                        .collect(),
                })
            },
            Err(err) => {
                Err(PatternIndexBuildError::Option(format!("{}", err)))
            },
        }
    }
    fn locate(&self, pattern: &[u8], search_range: &Vec<u32>) -> Vec<PatternLocation> {
        let mut location = self.inner.locate(pattern);
        location.sort();
        sorted_positions_to_pattern_location(
            &location,
            &self.boundaries,
            search_range,
            pattern.len() as u32,
        )
    }
}
impl<B: Block<u64>> PatternIndex for Lfi64<B> {
    type Option = LfiOption;

    fn new(
        concatenated_sequence_with_boundaries: ConcatenatedSequenceWithBoundaries,
        sequence_type: &SequenceType,
        option: Self::Option,
    ) -> Result<Self, PatternIndexBuildError> {
        let mut valid_characters: Vec<Vec<u8>> = sequence_type.valid_characters()
            .into_iter().map(|v| vec![v]).collect();
        valid_characters.pop(); // Remove last character
        let characters_by_index: Vec<&[u8]> = valid_characters.iter()
            .map(|v| v.as_slice())
            .collect();
        match LtFmIndex::build(
            concatenated_sequence_with_boundaries.concatenated_sequence,
            &characters_by_index,
            option.suffix_array_sampling_ratio as u64,
            option.lookup_table_kmer_size,
        ) {
            Ok(v) => {
                Ok(Self {
                    inner: v,
                    boundaries: concatenated_sequence_with_boundaries.boundaries,
                })
            },
            Err(err) => {
                Err(PatternIndexBuildError::Option(format!("{}", err)))
            },
        }
    }
    fn locate(&self, pattern: &[u8], search_range: &Vec<u32>) -> Vec<PatternLocation> {
        let mut location = self.inner.locate(pattern);
        location.sort();
        sorted_positions_to_pattern_location(
            &location,
            &self.boundaries,
            search_range,
            pattern.len() as u32,
        )
    }
}

// Features
//  - Serialize
use crate::reference::features::Serialize;
use capwriter::{Save, Load};
impl<B: Block<u32>> Serialize for Lfi32<B> {
    fn save_to<W>(&self, mut writer: W) -> Result<(), std::io::Error> where
        W: std::io::Write
    {
        self.inner.save_to(&mut writer)?;
        self.boundaries.save_to(&mut writer)?;
        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self, std::io::Error> where
        R: std::io::Read,
        Self: Sized
    {
        let inner = LtFmIndex::load_from(&mut reader)?;
        let boundaries = Vec::load_from(&mut reader)?;
        Ok(Self { inner, boundaries })
    }
}
impl<B: Block<u64>> Serialize for Lfi64<B> {
    fn save_to<W>(&self, mut writer: W) -> Result<(), std::io::Error> where
        W: std::io::Write
    {
        self.inner.save_to(&mut writer)?;
        self.boundaries.save_to(&mut writer)?;
        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self, std::io::Error> where
        R: std::io::Read,
        Self: Sized
    {
        let inner = LtFmIndex::load_from(&mut reader)?;
        let boundaries = Vec::load_from(&mut reader)?;
        Ok(Self { inner, boundaries })
    }
}