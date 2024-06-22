use std::fmt::Debug;

pub trait AllocationStrategy: Debug + Clone {
    fn new() -> Self;
    fn initial_query_len(&self) -> u32;
    fn enlarge_query_len(&self, required_query_length: u32) -> u32;
}

#[derive(Clone)]
pub struct QueryLengthChecker<A: AllocationStrategy> {
    allocated_query_len: u32,
    allocation_strategy: A,
}
impl<A: AllocationStrategy> QueryLengthChecker<A> {
    pub fn new(allocation_strategy: A) -> Self {
        Self {
            allocated_query_len: allocation_strategy.initial_query_len(),
            allocation_strategy,
        }
    }
    #[inline(always)]
    pub fn get_allocated_length(&self) -> u32 {
        self.allocated_query_len
    }
    /// `None` if the allocated length is enough, otherwise `Some(new_length)``
    #[inline(always)]
    pub fn optional_length_to_be_allocated(
        &mut self,
        query_length: u32,
    ) -> Option<u32> {
        if self.allocated_query_len >= query_length {
            None
        } else {
            let new_length = self.allocation_strategy.enlarge_query_len(query_length);
            self.allocated_query_len = new_length;
            Some(new_length)
        }
    }
}

impl<A: AllocationStrategy> Debug for QueryLengthChecker<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryLengthChecker")
            .field("allocated", &self.allocated_query_len)
            .field("allocation_strategy", &self.allocation_strategy)
            .finish()
    }
}
