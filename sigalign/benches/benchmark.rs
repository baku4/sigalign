use criterion::{
    criterion_group, criterion_main,
};

mod test_data;

mod to_json;
use to_json::{
    bench_to_json_pretty
};

criterion_group!(
    benches,
    bench_to_json_pretty,
);
criterion_main!(benches);