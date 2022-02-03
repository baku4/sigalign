use criterion::{
    black_box, criterion_group, criterion_main, Criterion, BenchmarkId
};

use sigalign::result::FastaAlignmentLabeledResult;

use super::test_data::get_test_alignment_result;

use serde_json::{
    to_string,
    to_string_pretty,
};

#[inline]
fn to_json_with_serde(result: &FastaAlignmentLabeledResult) -> String {
    to_string(&result).unwrap()
}

#[inline]
fn to_json_with_serde_pretty(result: &FastaAlignmentLabeledResult) -> String {
    to_string_pretty(&result).unwrap()
}


/*
serde json pretty
*/

pub fn bench_to_json_pretty(c: &mut Criterion) {
    let mut group = c.benchmark_group("to_json_pretty");

    let test_result = get_test_alignment_result();

    group.bench_function(
        "serde",
        |b| b.iter(|| {
            to_json_with_serde(black_box(&test_result));
        })
    );

    group.bench_function(
        "serde_pretty",
        |b| b.iter(|| {
            to_json_with_serde_pretty(black_box(&test_result));
        })
    );

    group.finish();
}
