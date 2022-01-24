// Alignment conditions that affect the alignment result

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Penalties {
    pub x: usize,
    pub o: usize,
    pub e: usize,
}

pub const PRECISION_SCALE: usize = 10_000; // Ensuring accuracy to the third decimal place.

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
