/// Returns (MinL, MaxP) with throughput constant.
///
/// For the throughput constant (λ), length of query (l) and mismatch penalty (x):
/// - MinL: ⌊λ * √l + 20⌋
///    - when ⌊λ * √l + 20⌋ > l, use l as MinL
/// - MaxP: x / (2 * λ * √l)
/// 
/// Suggested range for throughput constant:
/// 1 ≤ λ ≤ 5
///
/// Larger values -> Stricter cutoffs -> Faster the algorithm, but less sensitive
pub fn cutoffs_from_throughput_constant(
    mismatch_penalty: u32,
    length_of_query: u32,
    throughput_constant: f32,
) -> (u32, f32) {
    let min_length = ((
        throughput_constant * (length_of_query as f32).sqrt()
        + 20.0
    ) as u32).min(length_of_query);
    let max_penalty = (
        mismatch_penalty
    ) as f32 / (
        2.0 * throughput_constant * (length_of_query as f32).sqrt()
    );

    (min_length, max_penalty)
}

// TODO: remove when the function is stable
#[test]
fn print_cutoffs_from_throughput_constant() {
    for length in [30, 50, 100, 200, 400, 800, 1600, 3200, 6400] {
        println!(
            "Length: {}, Strict, MinL: {}, MaxP: {}",
            length,
            5 * (length as f32).sqrt() as u32,
            0.5 / (length as f32).sqrt(),
        );
        println!(
            "Length: {}, Lenient, MinL: {}, MaxP: {}",
            length,
            2 * (length as f32).sqrt() as u32,
            1.5 / (length as f32).sqrt(),
        );

        for lambda in 1..6 {
            let (min_length, max_penalty) = cutoffs_from_throughput_constant(
                4,
                length,
                lambda as f32,
            );
            println!("Length: {}, Lambda: {}, MinL: {}, MaxP: {}", length, lambda, min_length, max_penalty);
        }
    }
}
