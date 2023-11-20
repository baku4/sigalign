use crate::core::regulators::Penalty;
use super::{
    WaveFront, WaveFrontPool,
    safe_max_penalty_from_len,
};
#[derive(Clone)]
pub struct SingleWaveFrontPool {
    pub wave_front: WaveFront,
}
impl WaveFrontPool for SingleWaveFrontPool {
    fn new(
        query_len: u32,
        maximum_scaled_penalty_per_length: u32,
        penalties: &Penalty,
    ) -> Self {
        let max_penalty = safe_max_penalty_from_len(query_len, maximum_scaled_penalty_per_length, penalties);
        let wave_front = WaveFront::new_allocated(penalties, max_penalty as usize);
        Self {
            wave_front: wave_front,
        }
    }
    fn allocate(
        &mut self,
        query_len: u32,
        maximum_scaled_penalty_per_length: u32,
        penalties: &Penalty,
    ) {
        let max_penalty = safe_max_penalty_from_len(query_len, maximum_scaled_penalty_per_length, penalties);
        // TODO: not to allocate whole space.
        let wave_front = WaveFront::new_allocated(penalties, max_penalty as usize);
        self.wave_front = wave_front;
    }
}

impl std::fmt::Debug for SingleWaveFrontPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SingleWaveFrontPool")
            .field("num_wave_front_pool", b"1")
            .finish()
    }
}
