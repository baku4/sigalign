//! Alignment regulators
pub mod pattern_size;
pub use pattern_size::calculate_max_pattern_size;

pub const PREC_SCALE: u32 = 100_000; // Ensuring accuracy to the fourth decimal place.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Penalty {
    pub x: u32,
    pub o: u32,
    pub e: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cutoff {
    pub minimum_length: u32,
    pub maximum_scaled_penalty_per_length: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MinPenaltyForPattern {
    pub odd: u32,
    pub even: u32,
}

impl MinPenaltyForPattern {
    pub fn new(penalties: &Penalty) -> Self {
        let odd: u32;
        let even: u32;
        if penalties.x <= penalties.o + penalties.e {
            odd = penalties.x;
            if penalties.x * 2 <= penalties.o + (penalties.e * 2) {
                even = penalties.x;
            } else {
                even = penalties.o + (penalties.e * 2) - penalties.x;
            }
        } else {
            odd = penalties.o + penalties.e;
            even = penalties.e;
        }
        Self {
            odd,
            even
        }
    }
}
