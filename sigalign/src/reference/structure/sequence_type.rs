use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};

#[derive(Debug)]
pub enum SequenceType {
    NucleotideOnly,
    NucleotideWithNoise(u8),
    AminoAcidOnly,
    AminoacidWithNoise(u8),
}

impl SequenceType {
    pub fn is_nucleotide(&self) -> bool {
        match self {
            Self::NucleotideOnly | Self::NucleotideWithNoise(_) => true,
            Self::AminoAcidOnly | Self::AminoacidWithNoise(_) => false,
        }
    }
    pub fn with_noise(&self) -> bool {
        match self {
            Self::NucleotideOnly | Self::AminoAcidOnly => true,
            Self::NucleotideWithNoise(_) | Self::AminoacidWithNoise(_) => false,
        }
    }
}