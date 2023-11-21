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

criterion_group!(
    benches,
    read_fasta_file,
);
criterion_main!(benches);