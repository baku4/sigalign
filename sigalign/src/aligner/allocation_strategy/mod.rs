use std::marker::PhantomData;

pub trait AllocationStrategy: Clone {
    const INITIAL_QUERY_LEN: u32;
    fn get_enlarged_query_len(needed_query_len: u32) -> u32;
}
#[derive(Debug, Clone)]
pub struct LinearStrategy;
impl AllocationStrategy for LinearStrategy {
    const INITIAL_QUERY_LEN: u32 = 200;
    #[inline(always)]
    fn get_enlarged_query_len(needed_query_len: u32) -> u32 {
        needed_query_len + Self::INITIAL_QUERY_LEN
    }
}
#[derive(Debug, Clone)]
pub struct DoublingStrategy;
impl AllocationStrategy for DoublingStrategy {
    const INITIAL_QUERY_LEN: u32 = 200;
    #[inline(always)]
    fn get_enlarged_query_len(needed_query_len: u32) -> u32 {
        let leading_zeros = needed_query_len.leading_zeros();
        1 << (32 - leading_zeros)
    }
}

#[derive(Debug, Clone)]
pub struct QueryLengthChecker<A: AllocationStrategy> {
    allocated_query_len: u32,
    phantom: PhantomData<A>,
}
impl<A: AllocationStrategy> QueryLengthChecker<A> {
    pub fn new() -> Self {
        Self {
            allocated_query_len: A::INITIAL_QUERY_LEN,
            phantom: PhantomData,
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
            let new_length = A::get_enlarged_query_len(needed_query_len);
            self.allocated_query_len = new_length;
            Some(new_length)
        }
    }
}