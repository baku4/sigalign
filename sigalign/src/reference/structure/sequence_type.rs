use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};

use super::FixedSizeEncoding;
use std::io::{Write, Read};

#[derive(Debug)]
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

impl FixedSizeEncoding for SequenceType {
    const BYTE_SIZE_OF_STRUCTURE: usize = 2;

    fn write_to<W>(&self, mut writer: W) -> Result<()> where
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
        if result == Self::BYTE_SIZE_OF_STRUCTURE {
            Ok(())
        } else {
            error_msg!("Failed to write reference sequence type")
        }
    }
    fn read_from<R>(mut reader: R) -> Result<Self> where
        R: Read,
        Self: Sized,
    {
        let mut buffer = [0; Self::BYTE_SIZE_OF_STRUCTURE];

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
