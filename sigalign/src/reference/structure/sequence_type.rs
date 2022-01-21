use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};

use super::SizeAwareEncoding;
use std::io::{Write, Read};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceType {
    NucleotideOnly,
    NucleotideWithNoise(u8),
    AminoAcidOnly,
    AminoAcidWithNoise(u8),
}

impl SequenceType {
    pub fn is_nucleotide(&self) -> bool {
        match self {
            Self::NucleotideOnly | Self::NucleotideWithNoise(_) => true,
            Self::AminoAcidOnly | Self::AminoAcidWithNoise(_) => false,
        }
    }
    pub fn with_noise(&self) -> bool {
        match self {
            Self::NucleotideOnly | Self::AminoAcidOnly => true,
            Self::NucleotideWithNoise(_) | Self::AminoAcidWithNoise(_) => false,
        }
    }
}

const SEQUENCE_TYPE_SIZE: usize = 2;

impl SizeAwareEncoding for SequenceType {
    fn save_to<W>(&self, mut writer: W) -> Result<()> where
        W: Write,
    {
        let result = match self {
            Self::NucleotideOnly => {
                writer.write(&[0, 0])
            },
            Self::NucleotideWithNoise(noise) => {
                writer.write(&[1, *noise])
            },
            Self::AminoAcidOnly => {
                writer.write(&[2, 0])
            },
            Self::AminoAcidWithNoise(noise) => {
                writer.write(&[3, *noise])
            },
        }?;
        if result == SEQUENCE_TYPE_SIZE {
            Ok(())
        } else {
            error_msg!("Failed to write reference sequence type")
        }
    }
    fn load_from<R>(mut reader: R) -> Result<Self> where
        R: Read,
        Self: Sized,
    {
        let mut buffer = [0; SEQUENCE_TYPE_SIZE];

        reader.read_exact(&mut buffer)?;

        match buffer[0] {
            0 => {
                Ok(Self::NucleotideOnly)
            },
            1 => {
                Ok(Self::NucleotideWithNoise(buffer[1]))
            },
            2 => {
                Ok(Self::AminoAcidOnly)
            },
            3 => {
                Ok(Self::AminoAcidWithNoise(buffer[1]))
            },
            _ => {
                error_msg!("Failed to read reference sequence type")
            }
        }
    }
}
