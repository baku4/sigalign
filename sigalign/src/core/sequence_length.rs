use num_traits::PrimInt;

pub trait SeqLen:
    PrimInt
    + std::ops::AddAssign
    + std::ops::SubAssign
{
    const ZERO: Self;
    const ONE: Self;
    fn as_u32(self) -> u32;
    fn from_u32(value: u32) -> Self;
}

impl SeqLen for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[inline(always)]
    fn as_u32(self) -> u32 {
        self as u32
    }
    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }
}
impl SeqLen for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    #[inline(always)]
    fn as_u32(self) -> u32 {
        self as u32
    }
    #[inline(always)]
    fn from_u32(value: u32) -> Self {
        value as Self
    }
}