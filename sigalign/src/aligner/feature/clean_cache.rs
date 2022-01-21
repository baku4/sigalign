use crate::aligner::wave_front_cache::WaveFrontCache;

use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
    Aligner,
};

impl Aligner {
    pub fn clean_extension_cache(&mut self) {
        match self {
            Self::SemiGlobal(aligner) => {
                aligner.wave_front_cache.clean_cache(
                    &aligner.condition.penalties,
                    &aligner.condition.cutoff,
                );
            },
            Self::Local(aligner) => {
                aligner.wave_front_cache.clean_cache(
                    &aligner.condition.penalties,
                    &aligner.condition.cutoff,
                );
            },
        }
    }
}