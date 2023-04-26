use crate::core::regulators::{
    Penalty, Cutoff,
};
use super::{
    WaveFront, WaveFrontPool,
    safe_max_penalty_from_len,
};
#[derive(Clone)]
pub struct DoubleWaveFrontPool {
    pub wave_front_1: WaveFront,
    pub wave_front_2: WaveFront,
}
impl WaveFrontPool for DoubleWaveFrontPool {
    fn new(
        query_len: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    ) -> Self {
        let max_penalty = safe_max_penalty_from_len(query_len, penalties, cutoff);
        let wave_front_1 = WaveFront::new_allocated(penalties, max_penalty as usize);
        let wave_front_2 = wave_front_1.clone();
        Self {
            wave_front_1,
            wave_front_2,
        }
    }
    fn allocate(
        &mut self,
        query_length: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    ) {
        let max_penalty = safe_max_penalty_from_len(query_length, penalties, cutoff);
        let wave_front_1 = WaveFront::new_allocated(penalties, max_penalty as usize); // TODO: not to allocate whole space.
        let wave_front_2 = wave_front_1.clone();
        self.wave_front_1 = wave_front_1;
        self.wave_front_1 = wave_front_2;
    }
}

impl std::fmt::Debug for DoubleWaveFrontPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DoubleWaveFrontPool")
            .field("num_wave_front_pool", b"2")
            .finish()
    }
}
