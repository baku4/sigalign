#![allow(dead_code)]
#![allow(unused_imports)]
use criterion::{
    criterion_group, criterion_main,
};

// mod write_result_to_stdout;
// use write_result_to_stdout::write_result_to_stdout;
// mod transform_to_reverse_complement;
// use transform_to_reverse_complement::transform_to_reverse_complement;
mod count_matches;
use count_matches::count_the_consecutive_match;
mod fasta_reader;
use fasta_reader::read_fasta_file;
mod convert_results_to_string_with_itoa;
use convert_results_to_string_with_itoa::convert_results_to_tsv_string;

criterion_group!(
    benches,
    convert_results_to_tsv_string,
);
criterion_main!(benches);
