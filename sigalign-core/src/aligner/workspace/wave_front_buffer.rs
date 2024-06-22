use crate::core::regulators::{
    Penalty, PREC_SCALE,
};
use crate::algorithm::WaveFront;

#[derive(Clone)]
pub struct WaveFrontBuffer(WaveFront);

impl AsMut<WaveFront> for WaveFrontBuffer {
    fn as_mut(&mut self) -> &mut WaveFront {
        &mut self.0
    }
}

impl WaveFrontBuffer {
    pub fn new(
        query_length: u32,
        maximum_scaled_penalty_per_length: u32,
        penalties: &Penalty,
    ) -> Self {
        let max_penalty = safe_max_penalty_from_len(query_length, maximum_scaled_penalty_per_length, penalties);
        let wave_front = WaveFront::new_allocated(penalties, max_penalty as usize);
        Self(wave_front)
    }
    pub fn allocate(
        &mut self,
        query_length: u32,
        maximum_scaled_penalty_per_length: u32,
        penalties: &Penalty,
    ) {
        let max_penalty = safe_max_penalty_from_len(query_length, maximum_scaled_penalty_per_length, penalties);
        // TODO: not to allocate whole space.
        let wave_front = WaveFront::new_allocated(penalties, max_penalty as usize);
        self.0 = wave_front;
    }
}

#[inline(always)]
fn safe_max_penalty_from_len(
    query_len: u32,
    maximum_scaled_penalty_per_length: u32,
    penalties: &Penalty,
) -> u32 {
    u32::max(
        penalties.o,
        (
            maximum_scaled_penalty_per_length * (
                penalties.e * query_len - penalties.o
            )
        ) / (
            PREC_SCALE * penalties.e - maximum_scaled_penalty_per_length
        ) + 1
    )
}

impl std::fmt::Debug for WaveFrontBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaveFrontBuffer")
            .field("max_penalty", &self.0.max_penalty)
            .finish()
    }
}
