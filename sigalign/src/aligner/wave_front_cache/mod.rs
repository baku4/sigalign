use crate::core::{regulators::{
    Penalty, PREC_SCALE, Cutoff,
}};
use super::WaveFront;
use std::fmt;

pub trait WaveFrontCache {
    const QUERY_LEN_INC_UNIT: u32 = 200;

    fn new(penalties: &Penalty, cutoff: &Cutoff) -> Self;
    fn have_enough_space(&self, query_length: u32) -> bool;
    fn allocate_more_if_necessary(
        &mut self,
        query_length: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    );
    fn upper_spacious_query_length(query_length: u32) -> u32 {
        ((query_length / Self::QUERY_LEN_INC_UNIT) + 1) * Self::QUERY_LEN_INC_UNIT
    }
    fn clean_cache(&mut self, penalties: &Penalty, cutoff: &Cutoff);
}

const FIRST_ALLOCATED_QUERY_LENGTH: u32 = 200;

#[derive(Clone)]
pub struct SingleWaveFrontCache {
    pub allocated_query_length: u32,
    pub wave_front: WaveFront,
}
impl WaveFrontCache for SingleWaveFrontCache {
    fn new(penalties: &Penalty, cutoff: &Cutoff) -> Self {
        Self {
            allocated_query_length: FIRST_ALLOCATED_QUERY_LENGTH,
            wave_front: WaveFront::new_with_query_length(FIRST_ALLOCATED_QUERY_LENGTH, penalties, cutoff),
        }
    }
    fn have_enough_space(&self, query_length: u32) -> bool {
        self.allocated_query_length < query_length
    }
    // TODO: Not to make new wavefront
    fn allocate_more_if_necessary(&mut self, query_length: u32, penalties: &Penalty, cutoff: &Cutoff) {
        if self.allocated_query_length < query_length {
            let to_allocate_query_length = Self::upper_spacious_query_length(query_length);
            let allocated_wave_front = WaveFront::new_with_query_length(to_allocate_query_length, penalties, cutoff);
            
            self.allocated_query_length = to_allocate_query_length;
            self.wave_front = allocated_wave_front;
        }
    }
    fn clean_cache(&mut self, penalties: &Penalty, cutoff: &Cutoff) {
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
    pub allocated_query_length: u32,
    pub primary_wave_front: WaveFront,
    pub secondary_wave_front: WaveFront,
}
impl WaveFrontCache for DoubleWaveFrontCache {
    fn new(penalties: &Penalty, cutoff: &Cutoff) -> Self {
        let allocated_wave_front = WaveFront::new_with_query_length(FIRST_ALLOCATED_QUERY_LENGTH, penalties, cutoff);

        Self {
            allocated_query_length: FIRST_ALLOCATED_QUERY_LENGTH,
            primary_wave_front: allocated_wave_front.clone(),
            secondary_wave_front: allocated_wave_front,
        }
    }
    fn have_enough_space(&self, query_length: u32) -> bool {
        self.allocated_query_length < query_length
    }
    // TODO: Not to make new wavefront
    fn allocate_more_if_necessary(&mut self, query_length: u32, penalties: &Penalty, cutoff: &Cutoff) {
        if self.allocated_query_length < query_length {
            let to_allocate_query_length = Self::upper_spacious_query_length(query_length);
            let allocated_wave_front = WaveFront::new_with_query_length(to_allocate_query_length, penalties, cutoff);
            
            self.allocated_query_length = to_allocate_query_length;
            self.primary_wave_front = allocated_wave_front.clone();
            self.secondary_wave_front = allocated_wave_front;
        }
    }
    fn clean_cache(&mut self, penalties: &Penalty, cutoff: &Cutoff) {
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
        query_length: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    ) ->  Self {
        let max_penalty = Self::safe_max_penalty_from_length(query_length, penalties, cutoff);

        WaveFront::new_allocated(penalties, max_penalty as usize)
    }
    fn safe_max_penalty_from_length(
        query_length: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    ) -> u32 {
        let max_penalty = u32::max(
            penalties.o,
            (
                cutoff.maximum_penalty_per_scale * (
                    penalties.e * query_length - penalties.o
                )
            ) / (
                PREC_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
            ) + 1
        );
        max_penalty
    }
}