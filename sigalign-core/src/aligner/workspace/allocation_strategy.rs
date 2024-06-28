use std::fmt::Debug;
use super::AllocationStrategy;

#[derive(Clone)]
pub struct DefaultLinearStrategy;
impl AllocationStrategy for DefaultLinearStrategy {
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
impl Debug for DefaultLinearStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LinearStrategy")
    }
}

#[derive(Clone)]
pub struct DefaultDoublingStrategy;
impl AllocationStrategy for DefaultDoublingStrategy {
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
impl Debug for DefaultDoublingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DoublingStrategy")
    }
}

#[test]
fn check_if_enlarged_length_is_correct() {
    let strategy = DefaultLinearStrategy::new();
    assert_eq!(strategy.enlarge_query_len(100), 300);
    assert_eq!(strategy.enlarge_query_len(300), 500);
    assert_eq!(strategy.enlarge_query_len(500), 700);
    assert_eq!(strategy.enlarge_query_len(700), 900);
    assert_eq!(strategy.enlarge_query_len(900), 1100);

    let strategy = DefaultDoublingStrategy::new();
    assert_eq!(strategy.enlarge_query_len(100), 128);
    assert_eq!(strategy.enlarge_query_len(200), 256);
    assert_eq!(strategy.enlarge_query_len(256), 512);
    assert_eq!(strategy.enlarge_query_len(300), 512);
    assert_eq!(strategy.enlarge_query_len(500), 512);
    assert_eq!(strategy.enlarge_query_len(512), 1024);
    assert_eq!(strategy.enlarge_query_len(700), 1024);
    assert_eq!(
        strategy.enlarge_query_len((2_i32.pow(20)-1) as u32),
        (2_i32.pow(20)) as u32,
    );
    assert_eq!(
        strategy.enlarge_query_len(2_i32.pow(22) as u32),
        (2_i32.pow(23)) as u32,
    );
}
