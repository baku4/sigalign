use super::{
    WaveFrontCache,
    Aligner, Algorithms,
};

/// About workspace for extension
impl Aligner {
    /// Deallocate memory for workspace of alignment extension
    pub fn clean_extension_cache(&mut self) {
        match &mut self.algorithms {
            Algorithms::SemiGlobal(aligner) => {
                aligner.wave_front_cache.clean_cache(
                    &aligner.condition.penalties,
                    &aligner.condition.cutoff,
                );
            },
            Algorithms::Local(aligner) => {
                aligner.wave_front_cache.clean_cache(
                    &aligner.condition.penalties,
                    &aligner.condition.cutoff,
                );
            },
        }
    }
    /// Print extendable query length for allocated workspace
    pub fn print_query_length_for_cached_space(&self) -> usize {
        match &self.algorithms {
            Algorithms::SemiGlobal(aligner) => {
                aligner.wave_front_cache.allocated_query_length
            },
            Algorithms::Local(aligner) => {
                aligner.wave_front_cache.allocated_query_length
            },
        }
    }
}
