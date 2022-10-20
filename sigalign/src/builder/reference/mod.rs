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

/**
Safe builder for [Reference]

`Builder` has two configurations for [Reference]: (1) type and (2) index of sequences.

`Builder` needs [SequenceStorage] to build [Reference].

* * *
* Configurations
    1. `Type` of sequence
        - **Four** types are supported.
            - Nucleotide only
                - A, C, G, T
            - Nucleotide with noise
                - A, C, G, T + One unspecified character
            - Aminoacid only
                - A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y
            - Aminoacid with noise
                - A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y + One unspecified character
        - If type of sequence is not specified in `Builder`. Appropriate type is automatically detected from [SequenceStorage]. It is recommended in general cases.
        - Specification for type is useful when user wants to
            - (1) check if there are any exceptions to my [SequenceStorage].
            - (2) build [Reference] more fast.
    2. `Index` of sequence
        - There are **three** options for compression size.
            1. BWT block size
                - Whether the size of the BWT block is 64 or 128.
                - Default is using `64 sized block`.
                - The larger the size of the block, the more compressed the index is.
            2. Sampling ratio
                - Sampling ratio for suffix array.
                - Default is `2`.
                - The larger the sampling ratio, the more compressed the index is.
            3. Kmer size for lookup table
                - Kmer size for lookup table in `lt-fm-index`.
                    - Lookup table is pre-calculated count array.
                    - Using *k*-sized lookup table can skip first *k* operations for LF-mapping.
                - The size of kmer count array is $the number of supported type of sequence^kmer size=1$
                    - Since memory usage increases exponentially with the number of characters supported by the sequence type, it is not recommended to increase this value too much.
                - If type of sequence is not specified in `Builder`. Default value is determined by type of sequence.
                    - Nucleotide only: `8` - about 64KiB
                    - Nucleotide with noise: `7` - about 76KiB
                    - Aminoacid only: `4` - about 156KiB
                    - Aminoacid with noise: `4` - about 190KiB
                - The smaller the size of the block, the more compressed the index is.
        - The `index` is wrapper of `lt-fm-index`. More detailed descriptions for index is in the `lt-fm-index` library.
            - [Repository](https://github.com/baku4/lt-fm-index)
            - [Crate](https://crates.io/crates/lt-fm-index)
*/
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

/// Default methods
impl ReferenceBuilder {
    /// Make default builder
    pub fn new() -> Self {
        Self {
            sequence_type_option: SequenceTypeOption::default(),
            pattern_finder_option: PatternFinderOption::default(),
        }
    }
    /// Build reference with [SequenceStorage]
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
