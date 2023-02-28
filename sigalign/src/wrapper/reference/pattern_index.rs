use crate::{
    core::PatternLocation,
    reference::{
        sequence_type::SequenceType,
        pattern_index::{
            PatternIndex, PatternIndexBuildError, ConcatenatedSequenceWithBoundaries,
            implementations::lfi::{
                Lfi32B2V64, Lfi32B3V64, Lfi32B4V64, Lfi32B5V64,
                LfiOption,
            },
        },
        features::Serialize,
    }
};

pub enum LfiWrapper {
    B2(Lfi32B2V64),
    B3(Lfi32B3V64),
    B4(Lfi32B4V64),
    B5(Lfi32B5V64),
}
impl LfiWrapper {
    pub fn new_with_default_option(
        concatenated_sequence_with_boundaries: ConcatenatedSequenceWithBoundaries,
        sequence_type: &SequenceType,
    ) -> Result<Self, PatternIndexBuildError> {
        // Gen option for Lfi
        let chrs = sequence_type.valid_characters();
        let chr_count = chrs.len();
        let lookup_table_size = calculate_lookup_table_kmer_size(chr_count);
        let option = LfiOption {
            suffix_array_sampling_ratio: 1,
            lookup_table_kmer_size: lookup_table_size,
        };
        Self::new(concatenated_sequence_with_boundaries, sequence_type, option)
    }
}
impl PatternIndex for LfiWrapper {
    type Option = LfiOption;

    fn new(
        concatenated_sequence_with_boundaries: ConcatenatedSequenceWithBoundaries,
        sequence_type: &SequenceType,
        option: Self::Option,
    ) -> Result<Self, PatternIndexBuildError> {
        // Gen option for Lfi
        let chrs = sequence_type.valid_characters();
        let chr_count = chrs.len();
        if chr_count <= 4 {
            let inner = Lfi32B2V64::new(concatenated_sequence_with_boundaries, &sequence_type, option)?;
            Ok(Self::B2(inner))
        } else if chr_count <= 8 {
            let inner = Lfi32B3V64::new(concatenated_sequence_with_boundaries, &sequence_type, option)?;
            Ok(Self::B3(inner))
        } else if chr_count <= 16 {
            let inner = Lfi32B4V64::new(concatenated_sequence_with_boundaries, &sequence_type, option)?;
            Ok(Self::B4(inner))
        } else if chr_count <= 32 {
            let inner = Lfi32B5V64::new(concatenated_sequence_with_boundaries, &sequence_type, option)?;
            Ok(Self::B5(inner))
        } else {
            Err(PatternIndexBuildError::OverMaximumCharacters { max: 32, input: chr_count as u32 })
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

/// Calculate the kmer size that makes the lookup table smaller than 5Mb.
/// Maximum is 14
fn calculate_lookup_table_kmer_size(chr_count: usize) -> u32 {
    for v in 1..23 {
        let estimated_byte_size_of_lt = chr_count.pow(v);
        if estimated_byte_size_of_lt >= 5242880 { // 5Mb
            return v - 1
        }
    }
    22 // value of 2
}

use crate::core::{EndianType, WriteBytesExt, ReadBytesExt};
impl Serialize for LfiWrapper {
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
impl LfiWrapper {
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