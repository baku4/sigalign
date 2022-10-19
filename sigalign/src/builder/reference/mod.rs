use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceStorage, SequenceType, JoinedSequence, PatternFinder,
};

// About options
mod sequence_type_option;
mod pattern_finder_option;

/// Builder for `Reference`
#[derive(Debug, Clone)]
pub struct ReferenceBuilder {
    sequence_type_option: SequenceTypeOption, // `None` to infer
    pattern_finder_option: PatternFinderOption,
}

#[derive(Debug, Clone)]
struct SequenceTypeOption {
    sequence_type: Option<SequenceType>, // `None` to infer
}

impl Default for SequenceTypeOption {
    fn default() -> Self {
        Self {
            sequence_type: None,
        }
    }
}

#[derive(Debug, Clone)]
struct PatternFinderOption {
    sa_sampling_ratio: u64,
    kmer_size: Option<usize>, // `None` to recommend
    use_bwt_128: bool,
}

impl Default for PatternFinderOption {
    fn default() -> Self {
        Self {
            sa_sampling_ratio: 2,
            kmer_size: None,
            use_bwt_128: false,
        }
    }
}

impl ReferenceBuilder {
    pub fn new() -> Self {
        Self {
            sequence_type_option: SequenceTypeOption::default(),
            pattern_finder_option: PatternFinderOption::default(),
        }
    }
    pub fn build<S: SequenceStorage>(self, sequence_storage: S) -> Result<Reference<S>> {
        let joined_sequence = sequence_storage.get_joined_sequence();
        
        let sequence_type = match self.sequence_type_option.sequence_type {
            Some(sequence_type) => sequence_type,
            None => {
                SequenceType::infer_sequence_type_from_sequence(&joined_sequence.bytes)?
            },
        };

        let (is_nucleotide, with_noise) = match sequence_type {
            SequenceType::NucleotideOnly(_) => (true, false),
            SequenceType::NucleotideWithNoise(_) => (true, true),
            SequenceType::AminoAcidOnly(_) => (false, false),
            SequenceType::AminoAcidWithNoise(_) => (false, true),
        };

        let kmer_size_for_lookup_table = match self.pattern_finder_option.kmer_size {
            Some(v) => v,
            None => PatternFinderOption::default_kmer_size_for_count_array(&sequence_type),
        };

        let pattern_finder = PatternFinder::new(
            joined_sequence,
            is_nucleotide,
            with_noise,
            self.pattern_finder_option.use_bwt_128,
            self.pattern_finder_option.sa_sampling_ratio,
            kmer_size_for_lookup_table,
        )?;

        let target_record_index: Vec<u32> = (0..sequence_storage.total_record_count()).map(|v| v as u32).collect();

        Ok(Reference::new(
            sequence_type,
            pattern_finder,
            target_record_index,
            sequence_storage,
        ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inferring_sequence_type() {
        let only_nc = b"ACGTACGT";
        let st = SequenceType::infer_sequence_type_from_sequence(only_nc).unwrap();
        assert_eq!(st, SequenceType::new_nucleotide_only());

        let nc_with_noise_1 = b"ACGTNNNACGT";
        let st = SequenceType::infer_sequence_type_from_sequence(nc_with_noise_1).unwrap();
        assert_eq!(st, SequenceType::new_nucleotide_with_noise(b'N'));
        let nc_with_noise_2 = b"ACGT+++ACGT";
        let st = SequenceType::infer_sequence_type_from_sequence(nc_with_noise_2).unwrap();
        assert_eq!(st, SequenceType::new_nucleotide_with_noise(b'+'));

        let only_aa = b"ACDEFWYYY";
        let st = SequenceType::infer_sequence_type_from_sequence(only_aa).unwrap();
        assert_eq!(st, SequenceType::new_amino_acid_only());

        let aa_with_noise_1 = b"ACGT***WYYY";
        let st = SequenceType::infer_sequence_type_from_sequence(aa_with_noise_1).unwrap();
        assert_eq!(st, SequenceType::new_amino_acid_with_noise(b'*'));
        let aa_with_noise_2 = b"WYYY+++ACGT";
        let st = SequenceType::infer_sequence_type_from_sequence(aa_with_noise_2).unwrap();
        assert_eq!(st, SequenceType::new_amino_acid_with_noise(b'+'));

        let errored_seq_list = [
            b"AAAACGT+*".to_vec(),
            b"*+ACGT".to_vec(),
            b"+ACGT*".to_vec(),
            b"*ACWY+".to_vec(),
            b"*WYAC+".to_vec(),
            b"WY*AC+".to_vec(),
        ];

        for errored_seq in errored_seq_list {
            let st_res = SequenceType::infer_sequence_type_from_sequence(&errored_seq);
            assert!(st_res.is_err())
        }
    }
}
