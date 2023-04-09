use crate::core::{regulators::{
    Penalty, PREC_SCALE, Cutoff,
}};
use super::WaveFront;

pub trait WaveFrontPool {
    fn new(penalties: &Penalty, cutoff: &Cutoff) -> Self;
    fn allocate_if_needed(
        &mut self,
        query_length: u32,
        penalties: &Penalty,
        cutoff: &Cutoff,
    );
}
pub trait AllocationStrategy: Clone {
    const INITIAL_QUERY_LEN: u32;
    fn get_enlarged_query_len(needed_query_len: u32) -> u32;
}
#[derive(Debug, Clone)]
pub struct LinearStrategy;
impl AllocationStrategy for LinearStrategy {
    const INITIAL_QUERY_LEN: u32 = 200;
    fn get_enlarged_query_len(needed_query_len: u32) -> u32 {
        needed_query_len + Self::INITIAL_QUERY_LEN
    }
}
#[derive(Debug, Clone)]
pub struct DoublingStrategy;
impl AllocationStrategy for DoublingStrategy {
    const INITIAL_QUERY_LEN: u32 = 200;
    fn get_enlarged_query_len(needed_query_len: u32) -> u32 {
        let leading_zeros = needed_query_len.leading_zeros();
        1 << (32 - leading_zeros)
    }
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
