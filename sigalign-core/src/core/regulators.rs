//! Alignment regulators
use num::integer::div_ceil;

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


// For pattern size calculation
#[inline(always)]
pub fn calculate_max_pattern_size(
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    gap_extend_penalty: u32,
) -> u32 {
    // Initialize
    let mut lower_bound = get_lower_bound_of_n(
        0,
        cutoff.minimum_length,
        gap_extend_penalty,
        min_penalty_for_pattern.odd,
    );
    let mut upper_bound;
    let mut n = 0;
    let mut k = 0;
    let mut k1;
    let mut k2 = 0;
    
    while k < lower_bound {
        n += 1;
        upper_bound = lower_bound - 1;
        lower_bound = get_lower_bound_of_n(
            n,
            cutoff.minimum_length,
            gap_extend_penalty,
            min_penalty_for_pattern.odd,
        );

        if n % 2 == 1 { // If n is odd
            let m = (n+1) / 2;
            k1 = max_k_satisfying_maxp_for_odd(
                m,
                gap_extend_penalty,
                min_penalty_for_pattern.odd,
                min_penalty_for_pattern.even,
                cutoff.maximum_scaled_penalty_per_length,
            );
            k2 = max_k_satisfying_maxp_for_even(
                m,
                gap_extend_penalty,
                min_penalty_for_pattern.odd,
                min_penalty_for_pattern.even,
                cutoff.maximum_scaled_penalty_per_length,
            );
            k = upper_bound.min(k1).min(k2);
        } else {
            k = upper_bound.min(k2);
        }
    }
    k
}
#[inline(always)]
fn get_lower_bound_of_n(
    n: u32, // Number of patterns
    minl: u32, // Minimum length
    pe: u32, // Gap-extend penalty
    p1: u32, // Penalty for odd pattern
) -> u32 {
    let numerator = pe*(minl+4) - p1;
    let denominator = pe*(n+2);
    div_ceil(numerator, denominator) - 1
}
#[inline(always)]
fn max_k_satisfying_maxp_for_odd(
    m: u32,
    pe: u32,
    p1: u32,
    p2: u32,
    scaled_maxp: u32,
) -> u32 {
    let numerator = PREC_SCALE*pe*(m*p1+m*p2-p2) + 4*scaled_maxp*pe - scaled_maxp*p1;
    let denominator = scaled_maxp*pe*(2*m+1);
    div_ceil(numerator, denominator) - 2
}
#[inline(always)]
fn max_k_satisfying_maxp_for_even(
    m: u32,
    pe: u32,
    p1: u32,
    p2: u32,
    scaled_maxp: u32,
) -> u32 {
    let numerator = PREC_SCALE*m*pe*(p1+p2) + 4*scaled_maxp*pe - scaled_maxp*p1;
    let denominator = scaled_maxp*pe*(2*m+2);
    div_ceil(numerator, denominator) - 2
}

#[test]
fn calculate_max_pattern_size_without_panic() {
    let px = (1..10).collect::<Vec<u32>>();
    let po = (0..10).collect::<Vec<u32>>();
    let pe = (1..10).collect::<Vec<u32>>();
    let minl = (50..150).collect::<Vec<u32>>();
    let maxp = px.iter()
        .map(|&x| (x * (1..5).collect::<Vec<u32>>().iter().sum::<u32>()) as f32 / 100.0)
        .collect::<Vec<f32>>();

    for &px in px.iter() {
        for &po in po.iter() {
            for &pe in pe.iter() {
                for &minl in minl.iter() {
                    for &maxp in maxp.iter() {
                        let penalties = Penalty { x: px, o: po, e: pe };
                        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);
                        let cutoff = Cutoff { minimum_length: minl, maximum_scaled_penalty_per_length: (maxp * PREC_SCALE as f32) as u32 };
                        let _ = calculate_max_pattern_size(&cutoff, &min_penalty_for_pattern, pe);
                    }
                }
            }
        }
    }
}