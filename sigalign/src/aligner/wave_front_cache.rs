use super::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{WaveFront};
use std::fmt;

pub trait WaveFrontCache {
    const QUERY_LEN_INC_UNIT: usize = 150;

    fn new(penalties: &Penalties, cutoff: &Cutoff) -> Self;
    fn have_enough_space(&self, query_length: usize) -> bool;
    fn allocate_more_if_necessary(
        &mut self,
        query_length: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
    );
    fn upper_spacious_query_length(query_length: usize) -> usize {
        ((query_length / Self::QUERY_LEN_INC_UNIT) + 1) * Self::QUERY_LEN_INC_UNIT
    }
    fn clean_cache(&mut self, penalties: &Penalties, cutoff: &Cutoff);
}

const FIRST_ALLOCATED_QUERY_LENGTH: usize = 200;

#[derive(Clone)]
pub struct SingleWaveFrontCache {
    pub allocated_query_length: usize,
    pub wave_front: WaveFront,
}
impl WaveFrontCache for SingleWaveFrontCache {
    fn new(penalties: &Penalties, cutoff: &Cutoff) -> Self {
        Self {
            allocated_query_length: FIRST_ALLOCATED_QUERY_LENGTH,
            wave_front: WaveFront::new_with_query_length(FIRST_ALLOCATED_QUERY_LENGTH, penalties, cutoff),
        }
    }
    fn have_enough_space(&self, query_length: usize) -> bool {
        self.allocated_query_length < query_length
    }
    // TODO: Not to make new wavefront
    fn allocate_more_if_necessary(&mut self, query_length: usize, penalties: &Penalties, cutoff: &Cutoff) {
        if self.allocated_query_length < query_length {
            let to_allocate_query_length = Self::upper_spacious_query_length(query_length);
            let allocated_wave_front = WaveFront::new_with_query_length(to_allocate_query_length, penalties, cutoff);
            
            self.allocated_query_length = to_allocate_query_length;
            self.wave_front = allocated_wave_front;
        }
    }
    fn clean_cache(&mut self, penalties: &Penalties, cutoff: &Cutoff) {
        *self = Self::new(penalties, cutoff);
    }
}
impl fmt::Debug for SingleWaveFrontCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SingleWaveFrontCache")
            .field("allocated_query_length", &self.allocated_query_length)
            .finish()
    }
}

#[derive(Clone)]
pub struct DoubleWaveFrontCache {
    pub allocated_query_length: usize,
    pub primary_wave_front: WaveFront,
    pub secondary_wave_front: WaveFront,
}
impl WaveFrontCache for DoubleWaveFrontCache {
    fn new(penalties: &Penalties, cutoff: &Cutoff) -> Self {
        let allocated_wave_front = WaveFront::new_with_query_length(FIRST_ALLOCATED_QUERY_LENGTH, penalties, cutoff);

        Self {
            allocated_query_length: FIRST_ALLOCATED_QUERY_LENGTH,
            primary_wave_front: allocated_wave_front.clone(),
            secondary_wave_front: allocated_wave_front,
        }
    }
    fn have_enough_space(&self, query_length: usize) -> bool {
        self.allocated_query_length < query_length
    }
    // TODO: Not to make new wavefront
    fn allocate_more_if_necessary(&mut self, query_length: usize, penalties: &Penalties, cutoff: &Cutoff) {
        if self.allocated_query_length < query_length {
            let to_allocate_query_length = Self::upper_spacious_query_length(query_length);
            let allocated_wave_front = WaveFront::new_with_query_length(to_allocate_query_length, penalties, cutoff);
            
            self.allocated_query_length = to_allocate_query_length;
            self.primary_wave_front = allocated_wave_front.clone();
            self.secondary_wave_front = allocated_wave_front;
        }
    }
    fn clean_cache(&mut self, penalties: &Penalties, cutoff: &Cutoff) {
        *self = Self::new(penalties, cutoff);
    }
}
impl fmt::Debug for DoubleWaveFrontCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DoubleWaveFrontCache")
            .field("allocated_query_length", &self.allocated_query_length)
            .finish()
    }
}

// Safely WaveFront Allocation
impl WaveFront {
    fn new_with_query_length(
        query_length: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) ->  Self {
        let max_score = Self::safe_max_score_from_length(query_length, penalties, cutoff);

        WaveFront::new_allocated(penalties, max_score)
    }
    fn safe_max_score_from_length(
        query_length: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) -> usize {
        usize::max(
            penalties.o,
            (
                cutoff.maximum_penalty_per_scale * (
                    penalties.e * query_length - penalties.o
                )
            ) / (
                PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
            ) + 2
            // According to the formula, value 1 is sufficient for cap.
            // But since cap can be already added from previous calculations, it should be set to 2.
        )
    }
}