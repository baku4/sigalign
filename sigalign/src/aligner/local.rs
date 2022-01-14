use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{WaveFront};
use super::{AlignmentCondition};
use std::fmt;

pub struct LocalAligner {
    condition: AlignmentCondition,
    allocated_query_length: usize,
    primary_wave_front_cache: WaveFront,
    secondary_wave_front_cache: WaveFront,
}

impl AlignerInterface for LocalAligner {
    fn alignment(&mut self, reference: &mut dyn ReferenceInterface, query: Sequence) -> ReferenceAlignmentResult {
        
    }
}