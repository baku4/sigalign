use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

use crate::core::Sequence;

pub fn sorted_vec_using_rayon<I>(
    sequence_iterator: I,
    num_thread: u32,
) -> Vec<Sequence> where I: Iterator::<Item = Sequence> {
    let pool: rayon::ThreadPool = ThreadPoolBuilder::new().num_threads(num_thread as usize).build().unwrap();
    let mut seq_vec: Vec<Sequence> = sequence_iterator.collect();
    pool.install(|| {
        // Perform the sort within the context of the custom thread pool
        seq_vec.par_sort_unstable_by(
            |a, b| b.inner.len().cmp(&a.inner.len())
        );
    });
    seq_vec
}
