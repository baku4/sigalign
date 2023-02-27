use std::marker::PhantomData;

use crate::core::regulators::{
    Penalty, Cutoff,
};
use super::{
    WaveFront, WaveFrontPool, AllocationStrategy,
    safe_max_penalty_from_len,
};
#[derive(Clone)]
pub struct SingleWaveFrontPool<A: AllocationStrategy> {
    pub allocated_query_len: u32,
    pub wave_front: WaveFront,
    phantom: PhantomData<A>,
}
impl<A: AllocationStrategy> WaveFrontPool for SingleWaveFrontPool<A> {
    fn new(penalties: &crate::core::regulators::Penalty, cutoff: &crate::core::regulators::Cutoff) -> Self {
        let query_len = A::INITIAL_QUERY_LEN;
        let max_penalty = safe_max_penalty_from_len(query_len, penalties, cutoff);
        let wave_front = WaveFront::new_allocated(penalties, max_penalty as usize);
        Self {
            allocated_query_len: query_len,
            wave_front: wave_front,
            phantom: PhantomData,
        }
    }
    fn allocate_if_needed(
        &mut self,
        query_len: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    ) {
        if self.allocated_query_len < query_len {
            let enlarged_query_len = A::get_enlarged_query_len(query_len);
            let max_penalty = safe_max_penalty_from_len(enlarged_query_len, penalties, cutoff);
            let wave_front = WaveFront::new_allocated(penalties, max_penalty as usize); // TODO: not to allocate whole space.
            self.allocated_query_len = enlarged_query_len;
            self.wave_front = wave_front;
        }
    }
}

impl<A: AllocationStrategy> std::fmt::Debug for SingleWaveFrontPool<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleWaveFrontPool")
            .field("allocated_query_len", &self.allocated_query_len)
            .field("allocation_strategy", &self.phantom)
            .field("num_wave_front_pool", b"1")
            .finish()
    }
}

