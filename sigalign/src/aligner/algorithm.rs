// Alignment algorithms
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

mod common_steps_dep;
use common_steps_dep::{AlignmentHashSet};

mod semi_global_dep;
mod local_dep;

pub use local_dep::local_alignment_algorithm_dep;
pub use semi_global_dep::semi_global_alignment_algorithm_dep;

// New version!
mod common_steps;
pub use common_steps::WaveFront;
use common_steps::{
    PosTable, AnchorPosition, AnchorIndex,
    calculate_spare_penalty,
    Extension, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker,
    TraversedPosition, TraversedAnchor,
};

mod local;
pub use local::local_alignment_algorithm;
mod semi_global;
pub use semi_global::semi_global_alignment_algorithm;

#[cfg(test)]
mod tests {
    use crate::aligner::wave_front_cache;
    use crate::{sequence_provider::*, ReferenceBuilder, Aligner};
    use super::*;
    use crate::tests::sample_data::{NUCLEOTIDE_ONLY_FA_PATH_1, NUCLEOTIDE_ONLY_FA_PATH_2, SIMPLE_FA_PATH};
    use crate::util::FastaReader;

    use std::time::{Duration, Instant};
    use std::thread::sleep;

    #[test]
    fn print_time_dep_vs_new() {
        // Reference
        let mut sequence_provider = InMemoryProvider::new();
        sequence_provider.add_fasta_file(NUCLEOTIDE_ONLY_FA_PATH_1).unwrap();
        let reference = ReferenceBuilder::new().build(sequence_provider).unwrap();
        let mut sequence_buffer = reference.get_buffer();

        // Alignment conditions
        let penalties = Penalties {
            x: 5,
            o: 6,
            e: 3,
        };
        let cutoff = Cutoff {
            minimum_aligned_length: 50,
            maximum_penalty_per_scale: 500,
        };
        let min_penalty_for_pattern = MinPenaltyForPattern { odd: 6, even: 5 };
        let pattern_size = 25;

        let mut left_wave_front = WaveFront::new_allocated(&penalties, 2000);
        let mut right_wave_front = WaveFront::new_allocated(&penalties, 2000);

        // Pos Table
        let fasta_reader = FastaReader::from_file_path(NUCLEOTIDE_ONLY_FA_PATH_2).unwrap();

        for (label, query) in fasta_reader {
            println!("# {}", label);

            // LOCAL

            let start = Instant::now();
            let mut result_dep = local_dep::local_alignment_algorithm_dep(
                &reference,
                &mut sequence_buffer,
                &query,
                pattern_size,
                &penalties,
                &cutoff,
                &min_penalty_for_pattern,
                &mut left_wave_front,
                &mut right_wave_front,
            );
            result_dep.0.sort_by_key(|x| x.index);
            println!("lo-dep: {}, {}", result_dep.0.iter().map(|x| x.alignments.len()).sum::<usize>(), start.elapsed().as_secs_f64());

            let start = Instant::now();
            let mut result_new = local::local_alignment_algorithm(
                &reference,
                &mut sequence_buffer,
                &query,
                pattern_size,
                &penalties,
                &cutoff,
                &mut left_wave_front,
                &mut right_wave_front,
            );
            result_new.0.sort_by_key(|x| x.index);
            println!("lo-new: {}, {}", result_new.0.iter().map(|x| x.alignments.len()).sum::<usize>(), start.elapsed().as_secs_f64());

            // assert_eq!(result_dep, result_new);

            // SEMI-GLOBAL

            let start = Instant::now();
            let mut result_dep = semi_global_dep::semi_global_alignment_algorithm_dep(
                &reference,
                &mut sequence_buffer,
                &query,
                pattern_size,
                &penalties,
                &cutoff,
                &min_penalty_for_pattern,
                &mut left_wave_front,
            );
            result_dep.0.sort_by_key(|x| x.index);
            println!("sg-dep: {}, {}", result_dep.0.iter().map(|x| x.alignments.len()).sum::<usize>(), start.elapsed().as_secs_f64());

            let start = Instant::now();
            let mut result_new = semi_global::semi_global_alignment_algorithm(
                &reference,
                &mut sequence_buffer,
                &query,
                pattern_size,
                &penalties,
                &min_penalty_for_pattern,
                &cutoff,
                &mut left_wave_front,
            );
            result_new.0.sort_by_key(|x| x.index);
            println!("sg-new: {}, {}", result_new.0.iter().map(|x| x.alignments.len()).sum::<usize>(), start.elapsed().as_secs_f64());

            // assert_eq!(result_dep, result_new);
        }
    }
}