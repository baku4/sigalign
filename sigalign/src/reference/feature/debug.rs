use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder, Serializable,
};

impl<S> Reference<S> where
    S: SequenceProvider,
{
    // Sequence Type
    pub fn get_whether_text_type_is_nucleotide(&self) -> bool {
        match &self.sequence_type {
            SequenceType::NucleotideOnly(_) | SequenceType::NucleotideWithNoise(_) => true,
            SequenceType::AminoAcidOnly(_) | SequenceType::AminoAcidWithNoise(_) => false,
        }
    }
    pub fn get_noise_character_of_text_type(&self) -> Option<u8> {
        match &self.sequence_type {
            SequenceType::NucleotideOnly(_) | SequenceType::AminoAcidOnly(_) => None,
            SequenceType::NucleotideWithNoise(chr_list) => {
                Some(*chr_list.last().unwrap())
            },
            SequenceType::AminoAcidWithNoise(chr_list) => {
                Some(*chr_list.last().unwrap())
            },
        }
    }
    pub fn get_allowed_character_list(&self) -> &[u8] {
        self.sequence_type.get_allowed_character_list()
    }
    // Pattern Finder
    pub fn get_suffix_array_sampling_ratio(&self) -> u64 {
        self.pattern_finder.get_suffix_array_sampling_ratio()
    }
    pub fn get_lookup_table_kmer_size(&self) -> usize {
        self.pattern_finder.get_lookup_table_kmer_size()
    }
    pub fn get_size_of_bwt_block(&self) -> usize {
        self.pattern_finder.get_size_of_bwt_block()
    }
    // Sequence Provider
    pub fn total_record_count(&self) -> usize {
        self.sequence_provider.total_record_count()
    }
}