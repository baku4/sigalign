// Alignment conditions that affect the alignment result
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Penalties {
    pub x: usize,
    pub o: usize,
    pub e: usize,
}

#[cfg(not(feature = "four_dp"))]
pub const PRECISION_SCALE: usize = 10_000; // Ensuring accuracy to the third decimal place.
#[cfg(feature = "four_dp")]
pub const PRECISION_SCALE: usize = 100_000; // Ensuring accuracy to the fourth decimal place.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cutoff {
    pub minimum_aligned_length: usize,
    pub maximum_penalty_per_scale: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MinPenaltyForPattern {
    pub odd: usize,
    pub even: usize,
}
