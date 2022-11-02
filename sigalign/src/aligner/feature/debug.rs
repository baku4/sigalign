use super::{
    Aligner, Algorithms,
};

use std::fmt::Debug;

impl Debug for Aligner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let algorithm: &str;
        let cache_query_length: usize;

        let alignment_condition = match &self.algorithms {
            Algorithms::SemiGlobal(aligner) => {
                algorithm = "SemiGlobal";
                cache_query_length = aligner.wave_front_cache.allocated_query_length;
                &aligner.condition
            },
            Algorithms::Local(aligner) => {
                algorithm = "Local";
                cache_query_length = aligner.wave_front_cache.allocated_query_length;
                &aligner.condition
            },
        };

        f.debug_struct("Aligner")
            .field("algorithm", &algorithm)
            .field("alignment_condition", alignment_condition)
            .field("cache_query_length", &cache_query_length)
            .finish()
    }
}