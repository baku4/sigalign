use criterion::{
    black_box, Criterion, BenchmarkId,
    PlotConfiguration, AxisScale,
};
use sigalign_tests::get_stable_result_of_val_data;
use sigalign::results::{
    fasta::{
        FastaAlignmentResult,
        ReadAlignmentResult,
    },
    AlignmentResult,
    TargetAlignmentResult,
    AnchorAlignmentResult,
    AlignmentOperations,
    AlignmentOperation,
};

mod type1;
use type1::*;
mod type2;
use type2::*;

use sigalign_tests::random_text_and_pattern::{
    gen_rand_chr_list, gen_rand_text,
};
fn get_test_data(
    match_len: usize,
    side_unmatched_seq_len: usize,
) -> (Vec<u8>, Vec<u8>) {
    let chr_list = gen_rand_chr_list(4);
    let text = gen_rand_text(&chr_list, match_len, match_len);

    let a = vec![b'A'; side_unmatched_seq_len];
    let t = vec![b'T'; side_unmatched_seq_len];

    let mut qry_seq = a.clone();
    qry_seq.extend_from_slice(&text);
    qry_seq.extend_from_slice(&a);

    let mut tgt_seq = t.clone();
    tgt_seq.extend_from_slice(&text);
    tgt_seq.extend_from_slice(&t);

    (qry_seq, tgt_seq)
}

pub fn count_the_consecutive_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("count_the_consecutive_match");

    let match_len: usize = 1000;
    let side_unmatched_seq_len: usize = 250;
    
    let (qry_seq, tgt_seq) = get_test_data(match_len, side_unmatched_seq_len);

    group.bench_function(
        "count_forward_1",
        |b| b.iter(|| {
            count_forward_1(
                black_box(&tgt_seq),
                black_box(&qry_seq),
                black_box(side_unmatched_seq_len),
                black_box(side_unmatched_seq_len),
            );
        }
    ));
    group.bench_function(
        "count_forward_2",
        |b| b.iter(|| {
            count_forward_2(
                black_box(&tgt_seq),
                black_box(&qry_seq),
                black_box(side_unmatched_seq_len),
                black_box(side_unmatched_seq_len),
            );
        }
    ));
    group.bench_function(
        "count_backward_1",
        |b| b.iter(|| {
            count_backward_1(
                black_box(&tgt_seq),
                black_box(&qry_seq),
                black_box(match_len-side_unmatched_seq_len),
                black_box(match_len-side_unmatched_seq_len),
            );
        }
    ));
    group.bench_function(
        "count_backward_2",
        |b| b.iter(|| {
            count_backward_2(
                black_box(&tgt_seq),
                black_box(&qry_seq),
                black_box(match_len-side_unmatched_seq_len),
                black_box(match_len-side_unmatched_seq_len),
            );
        }
    ));
}
