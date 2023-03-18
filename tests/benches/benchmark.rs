use criterion::{
    criterion_group, criterion_main,
};

mod write_result_to_stdout;
use write_result_to_stdout::write_result_to_stdout;

criterion_group!(
    benches,
    write_result_to_stdout,
);
criterion_main!(benches);