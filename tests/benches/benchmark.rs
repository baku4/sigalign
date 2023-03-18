use criterion::{
    criterion_group, criterion_main,
};

mod write_result_to_stdout;
use write_result_to_stdout::write_result_to_stdout;
mod transform_to_reverse_complement;
use transform_to_reverse_complement::transform_to_reverse_complement;

criterion_group!(
    benches,
    transform_to_reverse_complement,
);
criterion_main!(benches);