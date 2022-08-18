use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider, SequenceType, JoinedSequence, PatternFinder,
};
use super::{
    ReferenceBuilder, SequenceTypeOption, PatternFinderOption,
};

impl ReferenceBuilder {
    pub fn change_suffix_array_sampling_ratio(mut self, sampling_ratio: u64) -> Result<Self> {
        if sampling_ratio <= 0 {
            error_msg!("Sampling ratio accept positive integer.")
        } else {
            self.pattern_finder_option.sa_sampling_ratio = sampling_ratio;
            Ok(self)
        }
    }
    pub fn change_count_array_kmer(mut self, kmer: usize) -> Result<Self> {
        if kmer < 2 {
            error_msg!("The size of the kmer cannot be less than 2")
        } else {
            self.pattern_finder_option.kmer_size = Some(kmer);
            Ok(self)
        }
    }
    pub fn change_bwt_vector_size_to_64(mut self) -> Self {
        self.pattern_finder_option.use_bwt_128 = false;
        self
    }
    pub fn change_bwt_vector_size_to_128(mut self) -> Self {
        self.pattern_finder_option.use_bwt_128 = true;
        self
    }
}

impl PatternFinderOption {
    pub fn default_kmer_size_for_count_array(sequence_type: &SequenceType) -> usize {
        match sequence_type {
            SequenceType::NucleotideOnly(_) => 8, // About 64 Kb for kmer count array
            SequenceType::NucleotideWithNoise(_) => 7, // About 76 Kb for kmer count array
            SequenceType::AminoAcidOnly(_) => 4, // About 156 Kb for kmer count array
            SequenceType::AminoAcidWithNoise(_) => 4, // About 190 Kb for kmer count array
        }
    }
}