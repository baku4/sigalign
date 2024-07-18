/// Returns cutoffs with throughput constant.
///
/// For the throughput constant (λ), length of query (l) and mismatch penalty (x):
/// - MinL: λ√l + 20
/// - MaxP: (λ * x)/(8 * √l)
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
    let min_length = (
        throughput_constant * (length_of_query as f32).sqrt()
        + 20.0
    ) as u32;
    let max_penalty = (
        throughput_constant * mismatch_penalty as f32
    ) / (
        8.0 * (length_of_query as f32).sqrt()
    );

    (min_length, max_penalty)
}
