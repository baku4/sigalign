use std::fmt::Debug;
use sigalign_core::aligner::AllocationStrategy;

#[derive(Clone)]
pub struct LinearStrategy;
impl AllocationStrategy for LinearStrategy {
    fn new() -> Self {
        Self
    }
    #[inline(always)]
    fn initial_query_len(&self) -> u32 {
        200
    }
    #[inline(always)]
    fn enlarge_query_len(&self, needed_query_len: u32) -> u32 {
        needed_query_len + 200
    }
}
impl Debug for LinearStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LinearStrategy")
    }
}

#[derive(Clone)]
pub struct DoublingStrategy;
impl AllocationStrategy for DoublingStrategy {
    fn new() -> Self {
        Self
    }
    #[inline(always)]
    fn initial_query_len(&self) -> u32 {
        200
    }
    #[inline(always)]
    fn enlarge_query_len(&self, needed_query_len: u32) -> u32 {
        let leading_zeros = needed_query_len.leading_zeros();
        1 << (32 - leading_zeros)
    }
}
impl Debug for DoublingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DoublingStrategy")
    }
}
