use num::integer::{div_ceil, div_floor};

use super::{Penalty, Cutoff, MinPenaltyForPattern, PREC_SCALE};

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
        cutoff.minimum_length + 2,
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
    let mut m = calculate_m(k, cutoff.minimum_length);
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
    let minimum_length = cutoff.minimum_length;

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
                        let _ = calculate_max_pattern_size(
                            &penalties,
                            &cutoff, &min_penalty_for_pattern,
                        );
                    }
                }
            }
        }
    }
}
