use criterion::{
    black_box, Criterion, BenchmarkId,
    PlotConfiguration, AxisScale,
};
mod type1;
use type1::{
    FastaReader1, count_seq_len_with_fasta_readers_1,
};
mod type2;
use type2::{
    FastaReader2, count_seq_len_with_fasta_readers_2,
};

use std::path::PathBuf;

fn get_test_fasta_path() -> PathBuf {
    use sigalign_tests::test_data_path::get_ref_for_val_path;

    let fasta_file = get_ref_for_val_path();
    
    fasta_file
}

pub fn read_fasta_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_fasta_file");

    let test_fasta_file = get_test_fasta_path();
    let repeat_for_file = 10;

    group.bench_function(
        "type_1",
        |b| b.iter(|| {
            count_seq_len_with_fasta_readers_1(
                black_box(
                    FastaReader1::new_repeat(test_fasta_file.clone(), repeat_for_file)
                ),
            )
        }
    ));

    group.bench_function(
        "type_2",
        |b| b.iter(|| {
            count_seq_len_with_fasta_readers_2(
                black_box(
                    FastaReader2::new_repeat(test_fasta_file.clone(), repeat_for_file)
                ),
            )
        }
    ));
}
