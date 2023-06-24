use crate::{
    reference::{
        extensions::{
            Serialize,
            EstimateSize,
        },
    },
    utils::get_unique_characters_of_sequence,
};

use super::{
    PatternIndex, PatternLocation, ConcatenatedSequenceWithBoundaries,
    PatternIndexBuildError,
    Lfi32B2V64, Lfi32B3V64, Lfi32B4V64, Lfi32B5V64,
    LfiOption,
};

pub enum DynamicLfi {
    B2(Lfi32B2V64),
    B3(Lfi32B3V64),
    B4(Lfi32B4V64),
    B5(Lfi32B5V64),
}
#[derive(Debug, Clone)]
pub struct DynamicLfiOption {
    pub suffix_array_sampling_ratio: u64,
    pub max_lookup_table_byte_size: usize,
}
impl DynamicLfiOption {
    fn translate_to_lfi_option(self, alignable_sequence: &[u8]) -> LfiOption {
        let chr_count = alignable_sequence.len();
        let lookup_table_size = calculate_lookup_table_kmer_size_using_safe_guard(
            chr_count,
            self.max_lookup_table_byte_size,
        );
        LfiOption {
            suffix_array_sampling_ratio: self.suffix_array_sampling_ratio,
            lookup_table_kmer_size: lookup_table_size,
            use_safe_guard: true,
        }
    }
}

impl PatternIndex for DynamicLfi {
    type Option = DynamicLfiOption;

    fn new(
        concatenated_sequence_with_boundaries: ConcatenatedSequenceWithBoundaries,
        option: Self::Option,
    ) -> Result<Self, PatternIndexBuildError> {
        let unique_characters = get_unique_characters_of_sequence(&concatenated_sequence_with_boundaries.concatenated_sequence);

        let lfi_option = option.translate_to_lfi_option(&unique_characters);
        let chr_count = unique_characters.len();

        if chr_count <= 3 {
            let inner = Lfi32B2V64::new(concatenated_sequence_with_boundaries, lfi_option)?;
            Ok(Self::B2(inner))
        } else if chr_count <= 7 {
            let inner = Lfi32B3V64::new(concatenated_sequence_with_boundaries, lfi_option)?;
            Ok(Self::B3(inner))
        } else if chr_count <= 15 {
            let inner = Lfi32B4V64::new(concatenated_sequence_with_boundaries, lfi_option)?;
            Ok(Self::B4(inner))
        } else if chr_count <= 31 {
            let inner = Lfi32B5V64::new(concatenated_sequence_with_boundaries, lfi_option)?;
            Ok(Self::B5(inner))
        } else {
            Err(PatternIndexBuildError::OverMaximumCharacters { max: 31, input: chr_count as u32 })
        }
    }
    fn locate(&self, pattern: &[u8], search_range: &Vec<u32>) -> Vec<PatternLocation> {
        match self {
            Self::B2(v) => v.locate(pattern, search_range),
            Self::B3(v) => v.locate(pattern, search_range),
            Self::B4(v) => v.locate(pattern, search_range),
            Self::B5(v) => v.locate(pattern, search_range),
        }
    }
}

/// Calculate the kmer size that makes the lookup table.
fn calculate_lookup_table_kmer_size_using_safe_guard(
    chr_count: usize,
    maximum_byte_size: usize,
) -> u32 {
    let max_cap = 50;
    for v in 1..=max_cap {
        let estimated_byte_size_of_lt = (chr_count+1).pow(v);
        if estimated_byte_size_of_lt >= maximum_byte_size {
            return v - 1
        }
    }
    max_cap
}

use crate::core::{EndianType, WriteBytesExt, ReadBytesExt};
impl Serialize for DynamicLfi {
    fn save_to<W>(&self, mut writer: W) -> Result<(), std::io::Error> where
        W: std::io::Write
    {
        match self {
            Self::B2(v) => {
                writer.write_u64::<EndianType>(Self::B2_MAGIC_NUMBER)?;
                v.save_to(&mut writer)?;
                Ok(())
            },
            Self::B3(v) => {
                writer.write_u64::<EndianType>(Self::B3_MAGIC_NUMBER)?;
                v.save_to(&mut writer)?;
                Ok(())
            },
            Self::B4(v) => {
                writer.write_u64::<EndianType>(Self::B4_MAGIC_NUMBER)?;
                v.save_to(&mut writer)?;
                Ok(())
            },
            Self::B5(v) => {
                writer.write_u64::<EndianType>(Self::B5_MAGIC_NUMBER)?;
                v.save_to(&mut writer)?;
                Ok(())
            },
        }
    }
    fn load_from<R>(mut reader: R) -> Result<Self, std::io::Error> where
        R: std::io::Read,
        Self: Sized,
    {
        let magic_number = reader.read_u64::<EndianType>()?;
        match magic_number {
            Self::B2_MAGIC_NUMBER => {
                let inner = Lfi32B2V64::load_from(&mut reader)?;
                Ok(Self::B2(inner))
            },
            Self::B3_MAGIC_NUMBER => {
                let inner = Lfi32B3V64::load_from(&mut reader)?;
                Ok(Self::B3(inner))
            },
            Self::B4_MAGIC_NUMBER => {
                let inner = Lfi32B4V64::load_from(&mut reader)?;
                Ok(Self::B4(inner))
            },
            Self::B5_MAGIC_NUMBER => {
                let inner = Lfi32B5V64::load_from(&mut reader)?;
                Ok(Self::B5(inner))
            },
            _ => {
                Err((std::io::ErrorKind::InvalidData).into())
            },
        }
    }
}
impl EstimateSize for DynamicLfi {
    fn serialized_size(&self) -> usize {
        std::mem::size_of::<u64>()
        + match self {
            Self::B2(v) => v.serialized_size(),
            Self::B3(v) => v.serialized_size(),
            Self::B4(v) => v.serialized_size(),
            Self::B5(v) => v.serialized_size(),
        }
    }
}
impl DynamicLfi {
    // MAGIC NUMBERS: FNV1A32 hash value of
    // LtFmIndexPosition32Block2Vector64: 956ed7f2
    const B2_MAGIC_NUMBER: u64 = 2507069426;
    // LtFmIndexPosition32Block3Vector64: 626c069d
    const B3_MAGIC_NUMBER: u64 = 1651246749;
    // LtFmIndexPosition32Block4Vector64: 6e317038
    const B4_MAGIC_NUMBER: u64 = 1848733752;
    // LtFmIndexPosition32Block5Vector64: 6a2427ab
    const B5_MAGIC_NUMBER: u64 = 1780754347;
}
