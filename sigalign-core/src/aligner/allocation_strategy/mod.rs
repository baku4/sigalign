use std::fmt::Debug;

pub trait AllocationStrategy: Debug + Clone {
    fn new() -> Self;
    fn initial_query_len(&self) -> u32;
    fn enlarge_query_len(&self, needed_query_len: u32) -> u32;
}

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

#[derive(Clone)]
pub(crate) struct QueryLengthChecker<A: AllocationStrategy> {
    allocated_query_len: u32,
    allocation_strategy: A,
}
impl<A: AllocationStrategy> QueryLengthChecker<A> {
    pub fn new() -> Self {
        let allocation_strategy = A::new();
        Self {
            allocated_query_len: allocation_strategy.initial_query_len(),
            allocation_strategy,
        }
    }
    #[inline(always)]
    pub fn get_allocated_length(&self) -> u32 {
        self.allocated_query_len
    }
    #[inline(always)]
    pub fn optional_length_to_be_allocated(
        &mut self,
        needed_query_len: u32,
    ) -> Option<u32> {
        if self.allocated_query_len >= needed_query_len {
            None
        } else {
            let new_length = self.allocation_strategy.enlarge_query_len(needed_query_len);
            self.allocated_query_len = new_length;
            Some(new_length)
        }
    }
}
impl<A: AllocationStrategy> Debug for QueryLengthChecker<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Query")
            .field("allocated", &self.allocated_query_len)
            .field("allocation_strategy", &self.allocation_strategy)
            .finish()
    }
}
