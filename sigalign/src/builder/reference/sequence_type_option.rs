use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceStorage, SequenceType, JoinedSequence, PatternFinder,
};
use super::{
    ReferenceBuilder, SequenceTypeOption, PatternFinderOption,
};

use std::ops::ControlFlow;

impl ReferenceBuilder {
    pub fn search_for_inferred_type_of_sequence(mut self, noise: u8) -> Self {
        self.sequence_type_option.sequence_type = None;
        self
    }
    pub fn search_for_nucleotide_only(mut self) -> Self {
        self.sequence_type_option.sequence_type = Some(SequenceType::new_nucleotide_only());
        self
    }
    pub fn search_for_nucleotide_with_noise(mut self, noise: u8) -> Self {
        self.sequence_type_option.sequence_type = Some(SequenceType::new_nucleotide_with_noise(noise));
        self
    }
    pub fn search_for_amino_acid_only(mut self) -> Self {
        self.sequence_type_option.sequence_type = Some(SequenceType::new_amino_acid_only());
        self
    }
    pub fn search_for_amino_acid_with_noise(mut self, noise: u8) -> Self {
        self.sequence_type_option.sequence_type = Some(SequenceType::new_amino_acid_with_noise(noise));
        self
    }
}

impl SequenceType {
    pub fn infer_sequence_type_from_sequence(reference_sequence: Sequence) -> Result<Self> {
        let mut can_be_nucleotide = true;
        let mut noise_of_nucleotide = None;
        let mut noise_of_amino_acid = None;

        let errored_chr = reference_sequence.iter().try_for_each(|&chr| {
            match chr {
                b'A' | b'C' | b'G' | b'T' => { // ACGT
                    // nothing to do
                    ControlFlow::Continue(())
                },
                b'D' | b'E' | b'F' | b'H' | b'I'
                | b'K' | b'L' | b'M' | b'N' | b'P'
                | b'Q' | b'R' | b'S' | b'V' | b'W' | b'Y' => { // Non ACGT Aminoacid (DEFHIKLMNPQRSVWY)
                    if can_be_nucleotide {
                        match noise_of_nucleotide {
                            Some(noise) => {
                                if noise != chr {
                                    can_be_nucleotide = false;
                                }
                            },
                            None => {
                                noise_of_nucleotide = Some(chr);
                            },
                        }
                    }
                    ControlFlow::Continue(())
                },
                _ => {
                    if can_be_nucleotide {
                        match noise_of_nucleotide {
                            Some(noise) => {
                                if noise != chr {
                                    can_be_nucleotide = false;
                                }
                            },
                            None => {
                                noise_of_nucleotide = Some(chr);
                            },
                        }
                    }
                    match noise_of_amino_acid {
                        Some(noise) => {
                            if noise != chr {
                                return ControlFlow::Break((chr, noise))
                            }
                        },
                        None => {
                            noise_of_amino_acid = Some(chr);
                        },
                    }
                    ControlFlow::Continue(())
                },
            }
        });

        match errored_chr {
            ControlFlow::Continue(_) => {
                if can_be_nucleotide {
                    match noise_of_nucleotide {
                        Some(noise) => Ok(SequenceType::new_nucleotide_with_noise(noise)),
                        None => Ok(SequenceType::new_nucleotide_only())
                    }
                } else {
                    match noise_of_amino_acid {
                        Some(noise) => Ok(SequenceType::new_amino_acid_with_noise(noise)),
                        None => Ok(SequenceType::new_amino_acid_only())
                    }
                }
            },
            ControlFlow::Break((v1, v2)) => {
                error_msg!("{} and {} cannot coexist in reference.", v1, v2)
            },
        }
    }
}