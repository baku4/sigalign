/// Alignment regulators
///   - Type is determined by machine's pointer width

pub const PREC_SCALE: u32 = 100_000; // Ensuring accuracy to the fourth decimal place.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Penalty {
    pub x: u32,
    pub o: u32,
    pub e: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cutoff {
    pub minimum_aligned_length: u32,
    pub maximum_scaled_penalty_per_length: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MinPenaltyForPattern {
    pub odd: u32,
    pub even: u32,
}
