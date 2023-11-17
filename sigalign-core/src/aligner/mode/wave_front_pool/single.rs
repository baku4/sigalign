use crate::core::regulators::{
    Penalty, Cutoff,
};
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
        penalties: &Penalty,
        cutoff: &Cutoff,
    ) -> Self {
        let max_penalty = safe_max_penalty_from_len(query_len, penalties, cutoff);
        let wave_front = WaveFront::new_allocated(penalties, max_penalty as usize);
        Self {
            wave_front: wave_front,
        }
    }
    fn allocate(
        &mut self,
        query_len: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    ) {
        let max_penalty = safe_max_penalty_from_len(query_len, penalties, cutoff);
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

