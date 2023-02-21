use num_traits::PrimInt;

pub trait SeqLen:
    PrimInt
    + std::ops::AddAssign
    + std::ops::SubAssign
{
    const ZERO: Self;
    const ONE: Self;
    fn from_u32(value: u32) -> Self;
    fn as_usize(self) -> usize;
    fn from_usize(value: usize) -> Self;
    fn as_i64(self) -> i64;
}

impl SeqLen for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }
    #[inline(always)]
    fn as_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }
    #[inline(always)]
    fn as_i64(self) -> i64 {
        self as _
    }
}
impl SeqLen for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }
    #[inline(always)]
    fn as_usize(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn from_usize(value: usize) -> Self {
        value as Self
    }
    #[inline(always)]
    fn as_i64(self) -> i64 {
        self as _
    }
}