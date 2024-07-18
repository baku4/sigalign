use crate::utils::get_unique_characters_of_sequence;
use super::static_lfi::{
    Lfi32B2V64,
    Lfi32B3V64,
    Lfi32B4V64,
    Lfi32B5V64,
    LfiOption,
};
// Re-export: The build error type is the same as the static version.
pub use super::static_lfi::LfiBuildError;
use sigalign_core::reference::PatternIndex;

/// The LtFmIndex that can adjust the type by the number of characters.
/// - The maximum number of characters that can be indexed is 31 (same as the `Lfi32B5V64`).
/// - The maximum length of one sequence is u32::MAX (same as the static version).
#[derive(Clone)]
pub enum DynamicLfi {
    B2(Lfi32B2V64),
    B3(Lfi32B3V64),
    B4(Lfi32B4V64),
    B5(Lfi32B5V64),
}

/// Option to define the structure of the LtFmIndex.
#[derive(Debug, Clone)]
pub struct DynamicLfiOption {
    pub suffix_array_sampling_ratio: u64,
    pub lookup_table_max_bytes_size: u64,
    pub use_safe_guard: bool,
}
impl DynamicLfiOption {
    fn to_lfi_option(self) -> LfiOption {
        LfiOption {
            suffix_array_sampling_ratio: self.suffix_array_sampling_ratio,
            lookup_table_max_bytes_size: self.lookup_table_max_bytes_size,
            use_safe_guard: self.use_safe_guard,
        }
    }
}

impl PatternIndex for DynamicLfi {
    type Option = DynamicLfiOption;
    type BuildError = LfiBuildError;

    fn new(
        concatenated_sequence: Vec<u8>,
        option: Self::Option,
    ) -> Result<Self, Self::BuildError> {
        let lfi_option = option.to_lfi_option();
        let unique_sequence = get_unique_characters_of_sequence(&concatenated_sequence);
        let chr_count = {
            if lfi_option.use_safe_guard {
                unique_sequence.len()
            } else {
                unique_sequence.len() - 1
            }
        };

        if chr_count <= 3 {
            let inner = Lfi32B2V64::new(concatenated_sequence, lfi_option)?;
            Ok(Self::B2(inner))
        } else if chr_count <= 7 {
            let inner = Lfi32B3V64::new(concatenated_sequence, lfi_option)?;
            Ok(Self::B3(inner))
        } else if chr_count <= 15 {
            let inner = Lfi32B4V64::new(concatenated_sequence, lfi_option)?;
            Ok(Self::B4(inner))
        } else if chr_count <= 31 {
            let inner = Lfi32B5V64::new(concatenated_sequence, lfi_option)?;
            Ok(Self::B5(inner))
        } else {
            Err(Self::BuildError::OverMaximumCharacters { max: 31, input: chr_count as u32 })
        }
    }
    fn get_sorted_positions(&self, pattern: &[u8]) -> Vec<u32> {
        match self {
            Self::B2(v) => v.get_sorted_positions(pattern),
            Self::B3(v) => v.get_sorted_positions(pattern),
            Self::B4(v) => v.get_sorted_positions(pattern),
            Self::B5(v) => v.get_sorted_positions(pattern),
        }
    }
}

// Impl Extensions
use sigalign_core::reference::extensions::{
    Serialize,
    EstimateSize,
};
//  - Serialize
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
//  - EstimateSize
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
