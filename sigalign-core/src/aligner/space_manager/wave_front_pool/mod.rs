use crate::core::regulators::{
    Penalty, PREC_SCALE,
};
use crate::algorithm::WaveFront;

pub trait WaveFrontPool {
    fn new(
        query_length: u32,
        maximum_scaled_penalty_per_length: u32,
        penalties: &Penalty,
    ) -> Self;
    fn allocate(
        &mut self,
        query_length: u32,
        maximum_scaled_penalty_per_length: u32,
        penalties: &Penalty,
    );
}

mod single;
pub use single::SingleWaveFrontPool;
mod double;
pub use double::DoubleWaveFrontPool;

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
