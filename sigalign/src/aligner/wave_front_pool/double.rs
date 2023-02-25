use std::marker::PhantomData;

use crate::core::regulators::{
    Penalty, Cutoff,
};
use super::{
    WaveFront, WaveFrontPool, AllocationStrategy,
    safe_max_penalty_from_len,
};
#[derive(Debug, Clone)]
pub struct DoubleWaveFrontPool<A: AllocationStrategy> {
    pub allocated_query_len: u32,
    pub wave_front_1: WaveFront,
    pub wave_front_2: WaveFront,
    phantom: PhantomData<A>,
}
impl<A: AllocationStrategy> WaveFrontPool for DoubleWaveFrontPool<A> {
    fn new(penalties: &crate::core::regulators::Penalty, cutoff: &crate::core::regulators::Cutoff) -> Self {
        let query_len = A::INITIAL_QUERY_LEN;
        let max_penalty = safe_max_penalty_from_len(query_len, penalties, cutoff);
        let wave_front_1 = WaveFront::new_allocated(penalties, max_penalty as usize);
        let wave_front_2 = wave_front_1.clone();
        Self {
            allocated_query_len: query_len,
            wave_front_1,
            wave_front_2,
            phantom: PhantomData,
        }
    }
    fn allocate_if_needed(
        &mut self,
        query_length: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    ) {
        if self.allocated_query_len < query_length {
            let next_query_len = A::next_query_len(self.allocated_query_len);
            let max_penalty = safe_max_penalty_from_len(next_query_len, penalties, cutoff);
            let wave_front_1 = WaveFront::new_allocated(penalties, max_penalty as usize);
            let wave_front_2 = wave_front_1.clone();
            self.allocated_query_len = next_query_len;
            self.wave_front_1 = wave_front_1;
            self.wave_front_1 = wave_front_2;
        }
    }
}
