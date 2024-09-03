use num::integer::{div_ceil, div_floor};

use crate::core::regulators::{Cutoff, MinPenaltyForPattern, Penalty, PREC_SCALE};

// For pattern size calculation
pub fn calculate_max_pattern_size(
    penalty: &Penalty,
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
) -> u32 {
    let mut lower_k = 1;
    let mut upper_k = upper_value_of_k(cutoff, min_penalty_for_pattern);

    let mut result = lower_k;

    while lower_k <= upper_k {
        let mid_k = lower_k + (upper_k - lower_k) / 2;

        if check_if_k_can_be_used_as_pattern_size(
            mid_k, penalty, cutoff, min_penalty_for_pattern
        ) {
            result = mid_k;
            lower_k = mid_k + 1;
        } else {
            upper_k = mid_k - 1;
        }
    }

    result
}

fn upper_value_of_k(
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
) -> u32 {
    let v1 = div_floor(
        PREC_SCALE * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even),
        2 * cutoff.maximum_scaled_penalty_per_length,
    );

    let v2 = div_ceil(
        cutoff.minimum_aligned_length + 2,
        2,
    ) - 1;

    v1.min(v2)
}

fn check_if_k_can_be_used_as_pattern_size(
    k: u32,
    penalty: &Penalty,
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
) -> bool {
    let mut m = calculate_m(k, cutoff.minimum_aligned_length);
    let optional_case_number = if m == 0 {
        m = 1;
        Some(1)
    } else {
        if_k_is_valid_when_the_length_is_minimum_length(
            k, m, penalty, cutoff, min_penalty_for_pattern
        )
    };
    match optional_case_number {
        Some(case_number) => {
            let next_points_are_valid = validate_next_five_points(case_number, k, m, penalty, cutoff, min_penalty_for_pattern);
            if next_points_are_valid {
                true
            } else {
                false
            }
        },
        None => false
    }
}

fn calculate_m(
    k: u32,
    minimum_length: u32,
) -> u32 {
    if k > minimum_length + 2 {
        return 0;
    }
    div_floor(minimum_length + 2 - k, 2 * k)
}

// Return "Case Number" when k is valid
fn if_k_is_valid_when_the_length_is_minimum_length(
    k: u32,
    m: u32,
    penalty: &Penalty,
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
) -> Option<u32> {
    let minimum_length = cutoff.minimum_aligned_length;

    let p_c: u32 = get_p_c(penalty);
    let (case_number, minimum_penalty) = if minimum_length == 2*m*k + k - 2 { // Case 1
        let minimum_penalty = m * min_penalty_for_pattern.odd + (m-1) * min_penalty_for_pattern.even;
        (1, minimum_penalty)
    } else if minimum_length == 2*m*k + k - 1 { // Case 2
        let minimum_penalty = m * min_penalty_for_pattern.odd + (m-1) * min_penalty_for_pattern.even
            + penalty.o + penalty.e - min_penalty_for_pattern.odd;
        (2, minimum_penalty)
    } else if minimum_length <= 2*m*k + 2*k - 2 { // Case 3
        let use_one_more_pattern = m * min_penalty_for_pattern.odd + m * min_penalty_for_pattern.even;
        let minimum_penalty = if minimum_length + 1 < 2*m*k + k {
            use_one_more_pattern
        } else {
            let from_previous = m * min_penalty_for_pattern.odd + (m-1) * min_penalty_for_pattern.even
                + penalty.o + penalty.e - min_penalty_for_pattern.odd
                + penalty.e * (minimum_length + 1 - 2*m*k - k);
            use_one_more_pattern.min(from_previous)
        };

        (3, minimum_penalty)
    } else if minimum_length == 2*m*k + 2*k - 1 { // Case 4
        let minimum_penalty = m * min_penalty_for_pattern.odd + m * min_penalty_for_pattern.even
            + penalty.o + penalty.e - min_penalty_for_pattern.even;
        (4, minimum_penalty)
    } else if minimum_length == 2*m*k + 2*k { // Case 5
        let minimum_penalty = m * min_penalty_for_pattern.odd + m * min_penalty_for_pattern.even
            + penalty.o + penalty.e - min_penalty_for_pattern.even
            + p_c;
        (5, minimum_penalty)
    } else { // Case 6: minimum_length < 3mk +2k - 2
        let use_one_more_pattern = (m+1) * min_penalty_for_pattern.odd + m * min_penalty_for_pattern.even;
        let minimum_penalty = if minimum_length < 2*m*k + 2*k {
            use_one_more_pattern
        } else {
            let from_previous = m * min_penalty_for_pattern.odd + m * min_penalty_for_pattern.even
                + penalty.o + penalty.e - min_penalty_for_pattern.even
                + p_c
                + penalty.e * (minimum_length - 2*m*k - 2*k);
            use_one_more_pattern.min(from_previous)
        };

        (6, minimum_penalty)
    };

    if penalty_and_length_are_not_touch_the_cutoff_line(minimum_penalty, minimum_length, cutoff) {
        Some(case_number)
    } else {
        None
    }
}

fn validate_next_five_points(
    case_number: u32,
    k: u32,
    m: u32,
    penalty: &Penalty,
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
) -> bool {
    // Check point 1
    {
        let mut l = 2*m*k + k - 2;
        let mut p = m * min_penalty_for_pattern.odd + (m-1) * min_penalty_for_pattern.even;
        if case_number > 1 {
            l += 2*k;
            p += min_penalty_for_pattern.odd + min_penalty_for_pattern.even;
        }

        if !penalty_and_length_are_not_touch_the_cutoff_line(p, l, cutoff) {
            return false;
        }
    }

    // Check point 2
    {
        let mut l = 2*m*k + k - 1;
        let mut p = m * min_penalty_for_pattern.odd + (m-1) * min_penalty_for_pattern.even
            + penalty.o + penalty.e - min_penalty_for_pattern.odd;
        if case_number > 2 {
            l += 2*k;
            p += min_penalty_for_pattern.odd + min_penalty_for_pattern.even;
        }

        if !penalty_and_length_are_not_touch_the_cutoff_line(p, l, cutoff) {
            return false;
        }
    }

    // Check point 3
    {
        let mut l = 2*m*k + 2*k - 2;
        let mut p = m * min_penalty_for_pattern.odd + m * min_penalty_for_pattern.even;
        if case_number > 3 {
            l += 2*k;
            p += penalty.o + penalty.e - min_penalty_for_pattern.even;
        }

        if !penalty_and_length_are_not_touch_the_cutoff_line(p, l, cutoff) {
            return false;
        }
    }

    // Check point 4
    {
        let mut l = 2*m*k + 2*k - 1;
        let mut p = m * min_penalty_for_pattern.odd + m * min_penalty_for_pattern.even
            + penalty.o + penalty.e - min_penalty_for_pattern.even;
        if case_number > 4 {
            l += 2*k;
            p += min_penalty_for_pattern.even;
        }

        if !penalty_and_length_are_not_touch_the_cutoff_line(p, l, cutoff) {
            return false;
        }
    }

    // Check point 5
    {
        let mut l = 2*m*k + 2*k;
        let mut p = m * min_penalty_for_pattern.odd + m * min_penalty_for_pattern.even
            + penalty.o + penalty.e - min_penalty_for_pattern.even
            + get_p_c(penalty);
        if case_number > 5 {
            l += 2*k;
            p += min_penalty_for_pattern.even;
        }

        if !penalty_and_length_are_not_touch_the_cutoff_line(p, l, cutoff) {
            return false;
        }
    }

    true
}

fn penalty_and_length_are_not_touch_the_cutoff_line(
    penalty: u32,
    length: u32,
    cutoff: &Cutoff,
) -> bool {
    if penalty * PREC_SCALE > cutoff.maximum_scaled_penalty_per_length * length {
        true
    } else {
        false
    }
}

fn get_p_c(
    penalty: &Penalty,
) -> u32 {
    if penalty.o + penalty.e <= penalty.x {
        0
    } else {
        penalty.e
    }
}

#[test]
fn div_floor_test() {
    assert_eq!(div_floor(10, 3), 3);
    assert_eq!(div_floor(9, 3), 3);
}


#[inline(always)]
pub fn calculate_max_pattern_size_old(
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    gap_extend_penalty: u32,
) -> u32 {
    // Initialize
    let mut lower_bound = get_lower_bound_of_n(
        0,
        cutoff.minimum_aligned_length,
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
            cutoff.minimum_aligned_length,
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

fn calculate_max_pattern_size_oldest(
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
) -> u32 {
    let mut m = 1;
    let mut upper_bound = div_ceil(
        cutoff.minimum_aligned_length + 4,
        2,
    ) - 2;
    loop {
        let lower_bound = (
            (cutoff.minimum_aligned_length + 4)  as f32 / (2*m + 2) as f32
            - 1_f32
        ).ceil() as u32;
        let max_penalty = div_ceil(
            PREC_SCALE * m * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even)
            + (4 * cutoff.maximum_scaled_penalty_per_length),
            2 * cutoff.maximum_scaled_penalty_per_length * (m+1)
        ) - 2;
        // println!("m: {}, lower_bound: {}, upper_bound: {}, max_penalty: {}", m, lower_bound, upper_bound, max_penalty);

        let pattern_size = max_penalty.min(upper_bound);
        if pattern_size >= lower_bound {
            return pattern_size as u32
        }
        m += 1;
        upper_bound = lower_bound - 1;
    }
}

#[test]
fn calculate_pattern_size_with_current_version() {
    for px in 1..=10 {
        for po in 1..=10 {
            for pe in 1..=10 {
                for x in 1..=30 {
                    for y in 1..=40 {
                        let minl = 20*x;
                        let maxp = 0.005 * y as f32;

                        println!("px: {}, po: {}, pe: {}, minl: {}, maxp: {}", px, po, pe, minl, maxp);
                        let penalties = Penalty { x: px, o: po, e: pe };
                        let cutoff = Cutoff { minimum_aligned_length: minl, maximum_scaled_penalty_per_length: (maxp * PREC_SCALE as f32) as u32 };
                        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);

                        let new_pattern_size = calculate_max_pattern_size(&penalties, &cutoff, &min_penalty_for_pattern);
                        let old_pattern_size = calculate_max_pattern_size_old(&cutoff, &min_penalty_for_pattern, pe);

                        println!("new: {}, old: {}", new_pattern_size, old_pattern_size);
                    }
                }
            }
        }
    }
}
