use super::{Serialize, EstimateSize};
use crate::core::{
    Sequence,
};
use std::io::{Write, Read, Error};
use std::ops::ControlFlow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceType {
    NucleotideOnly([u8; 4]),
    NucleotideWithNoise([u8; 5]),
    AminoAcidOnly([u8; 20]),
    AminoAcidWithNoise([u8; 21]),
}

impl SequenceType {
    pub fn new_nucleotide_only() -> Self {
        Self::NucleotideOnly([b'A', b'C', b'G', b'T'])
    }
    pub fn new_nucleotide_with_noise(noise: u8) -> Self {
        Self::NucleotideWithNoise([b'A', b'C', b'G', b'T', noise])
    }
    pub fn new_amino_acid_only() -> Self {
        Self::AminoAcidOnly([b'A', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'K', b'L', b'M', b'N', b'P', b'Q', b'R', b'S', b'T', b'V', b'W', b'Y'])
    }
    pub fn new_amino_acid_with_noise(noise: u8) -> Self {
        Self::AminoAcidWithNoise([b'A', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'K', b'L', b'M', b'N', b'P', b'Q', b'R', b'S', b'T', b'V', b'W', b'Y', noise])
    }
    pub fn searchable_chr_list(&self) -> &[u8] {
        match self {
            Self::NucleotideOnly(chr_list) => chr_list,
            Self::NucleotideWithNoise(chr_list) => chr_list,
            Self::AminoAcidOnly(chr_list) => chr_list,
            Self::AminoAcidWithNoise(chr_list) => chr_list,
        }
    }
    pub fn is_nucleotide(&self) -> bool {
        match self {
            Self::NucleotideOnly(_) | Self::NucleotideWithNoise(_) => true,
            Self::AminoAcidOnly(_) | Self::AminoAcidWithNoise(_) => false,
        }
    }
    pub fn with_noise(&self) -> bool {
        match self {
            Self::NucleotideOnly(_) | Self::AminoAcidOnly(_) => true,
            Self::NucleotideWithNoise(_) | Self::AminoAcidWithNoise(_) => false,
        }
    }
    pub fn searchable(&self, query: Sequence) -> bool {
        match self {
            Self::NucleotideOnly(chr_list) => {
                Self::query_is_in_character_list(query, chr_list)
            },
            Self::NucleotideWithNoise(chr_list) => {
                Self::query_is_in_character_list(query, chr_list)
            },
            Self::AminoAcidOnly(chr_list) => {
                Self::query_is_in_character_list(query, chr_list)
            },
            Self::AminoAcidWithNoise(chr_list) => {
                Self::query_is_in_character_list(query, chr_list)
            },
        }
    }
    pub fn get_allowed_character_list(&self) -> &[u8] {
        match self {
            Self::NucleotideOnly(v) => v,
            Self::NucleotideWithNoise(v) => v,
            Self::AminoAcidOnly(v) => v,
            Self::AminoAcidWithNoise(v) => v,
        }
    }
    fn query_is_in_character_list(query: Sequence, character_list: &[u8]) -> bool {
        let error = query.iter().try_for_each(|chr| {
            if !character_list.contains(chr) {
                return ControlFlow::Break(chr)
            }
            ControlFlow::Continue(())
        });

        match error {
            ControlFlow::Continue(_) => true,
            ControlFlow::Break(_) => false,
        }
    }
}

const SEQUENCE_TYPE_SIZE: usize = 2;

impl Serialize for SequenceType {
    fn save_to<W>(&self, mut writer: W) -> Result<(), Error> where
        W: Write,
    {
        let written_size = match self {
            Self::NucleotideOnly(_) => {
                writer.write(&[0, 0])
            },
            Self::NucleotideWithNoise(chr_list) => {
                writer.write(&[1, chr_list[4]])
            },
            Self::AminoAcidOnly(_) => {
                writer.write(&[2, 0])
            },
            Self::AminoAcidWithNoise(chr_list) => {
                writer.write(&[3, chr_list[20]])
            },
        }?;
        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self, Error> where
        R: Read,
        Self: Sized,
    {
        let mut buffer = [0; SEQUENCE_TYPE_SIZE];

        reader.read_exact(&mut buffer)?;

        match buffer[0] {
            0 => {
                Ok(Self::new_nucleotide_only())
            },
            1 => {
                Ok(Self::new_nucleotide_with_noise(buffer[1]))
            },
            2 => {
                Ok(Self::new_amino_acid_only())
            },
            3 => {
                Ok(Self::new_amino_acid_with_noise(buffer[1]))
            },
            _ => {
                error_msg!("Failed to read reference sequence type")
            }
        }
    }
}

impl EstimateSize for SequenceType {
    fn size_of(&self) -> usize {
        SEQUENCE_TYPE_SIZE
    }
}
