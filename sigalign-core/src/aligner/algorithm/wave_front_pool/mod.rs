use crate::core::regulators::{
    Penalty, PREC_SCALE, Cutoff,
};
use crate::algorithm::{
    WaveFront,
};

pub trait WaveFrontPool {
    fn new(
        query_length: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    ) -> Self;
    fn allocate(
        &mut self,
        query_length: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    );
}

mod single;
pub use single::SingleWaveFrontPool;
mod double;
pub use double::DoubleWaveFrontPool;

fn safe_max_penalty_from_len(
    query_len: u32,
    penalties: &Penalty,
    cutoff: &Cutoff,
) -> u32 {
    u32::max(
        penalties.o,
        (
            cutoff.maximum_scaled_penalty_per_length * (
                penalties.e * query_len - penalties.o
            )
        ) / (
            PREC_SCALE * penalties.e - cutoff.maximum_scaled_penalty_per_length
        ) + 1
    )
}
