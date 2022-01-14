use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{Extension, AlignmentHashSet, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty_from_determinant};

mod anchoring;
mod extending;
mod evaluating;


// ANCHOR


#[derive(Debug)]
pub struct Anchors {
    anchors: Vec<Anchor>,
}

#[derive(Debug)]
struct Anchor {
    query_position: usize,
    record_position: usize,
    size: usize,
    left_estimation: Estimation,
    right_estimation: Estimation,
    left_checkpoints: CheckPoints,
    right_checkpoints: CheckPoints,
    left_referable_extension: Option<ReferableExtension>,
    right_referable_extension: Option<ReferableExtension>,
    dropped: bool,
    connected_anchors: Vec<usize>,
}

#[derive(Debug)]
struct Estimation {
    penalty: usize,
    length: usize,
}

#[derive(Debug, Clone)]
enum ReferableExtension {
    Own(Extension),
    Ref(ExtensionReference),
}

impl ReferableExtension {
    fn penalty_and_length(&self) -> (usize, usize) {
        match self {
            Self::Own(extension) => {
                (extension.penalty, extension.length)
            },
            Self::Ref(extension_reference) => {
                (extension_reference.penalty, extension_reference.length)
            },
        }
    }
    fn insertion_and_deletion_count(&self) -> (u32, u32) {
        match self {
            Self::Own(extension) => {
                (extension.insertion_count, extension.deletion_count)
            },
            Self::Ref(extension_reference) => {
                (extension_reference.insertion_count, extension_reference.deletion_count)
            },
        }
    }
}

#[derive(Debug, Clone)]
struct ExtensionReference {
    penalty: usize,
    length: usize,
    insertion_count: u32,
    deletion_count: u32,
    operation_reference: OperationReference,
}

#[derive(Debug, Clone)]
struct OwnedOperations {
    operations: Vec<AlignmentOperation>,
}

#[derive(Debug, Clone)]
struct OperationReference {
    anchor_index: usize,
    start_point_of_operations: StartPointOfOperations,
}

#[derive(Debug, Clone)]
struct StartPointOfOperations {
    operation_index: usize,
    operation_count: u32,
}

#[derive(Debug)]
struct CheckPoints(Vec<CheckPoint>);

#[derive(Debug, Clone)]
struct CheckPoint {
    anchor_index: usize,
    anchor_size: u32,
    record_position_gap: u32,
    query_position_gap: u32,
}


// ALGORITHM


pub fn semi_global_alignment_algorithm(
    reference: &mut dyn ReferenceInterface,
    query: Sequence,
    pattern_size: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    primary_wave_front: &mut WaveFront,
) -> ReferenceAlignmentResult {
    let anchors_preset_by_record = Anchors::create_preset_by_record(reference, query, pattern_size);

    ReferenceAlignmentResult(
        anchors_preset_by_record.into_iter().filter_map(|(record_index, anchors_preset)| {
            let record_sequence = reference.sequence_of_record(record_index);
            let record_length = record_sequence.len();

            let mut anchors = Anchors::from_preset(anchors_preset, record_length, query, pattern_size, cutoff, penalties, min_penalty_for_pattern);

            anchors.extend(record_sequence, query, penalties, cutoff, primary_wave_front);

            let alignment_results = anchors.get_alignment_results_for_semi_global(cutoff);

            if alignment_results.len() == 0 {
                None
            } else {
                Some((record_index, alignment_results))
            }
        }).collect()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::reference::TestReference;

    fn max_kmer_satisfying_cutoff(cutoff: &Cutoff, min_penalty_for_pattern: &MinPenaltyForPattern) -> usize {
        let mut n = 1;
        loop {
            let upper_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n)  as f32 - 2_f32).ceil();
            let lower_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n + 2)  as f32 - 2_f32).ceil();
            let max_penalty = (
                (
                    (
                        (PRECISION_SCALE * n * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even))
                    )
                    + 4 * cutoff.maximum_penalty_per_scale
                ) as f32 / (2 * (n+1) * cutoff.maximum_penalty_per_scale) as f32
            ).ceil() - 2_f32;

            let kmer = max_penalty.min(upper_bound);
            #[cfg(test)]
            println!("#n {}\nu {}\nl {}\nmp {}\nk {}", n, upper_bound, lower_bound, max_penalty, kmer);

            if kmer >= lower_bound {
                return kmer as usize
            }
            n += 1;
        }
    }

    #[test]
    fn print_results_of_semi_global_alignment() {
        let mut test_reference = TestReference::new();

        let query = b"CGGATGCTCCGGCAGCCGACAGAACGAAGGATCTTGCCGGAAAATGAACTTCTGTTATTATTTTTGTGATTCA";

        let penalties = Penalties {x: 4, o: 5, e: 2};
        let cutoff = Cutoff {
            minimum_aligned_length: 30,
            maximum_penalty_per_scale: 3_000
        };
        let min_penalty_for_pattern = MinPenaltyForPattern { odd: 4, even: 3 };

        let kmer = max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);

        let mut left_wave_front = WaveFront::new_allocated(&penalties, 100);

        let alignment_results = semi_global_alignment_algorithm(
            &mut test_reference,
            query,
            kmer,
            &penalties,
            &cutoff,
            &min_penalty_for_pattern,
            &mut left_wave_front,
        );

        println!("alignment_results:\n{:#?}", alignment_results);
    }

    #[test]
    fn print_json_results_of_semi_global_alignment() {
        let mut test_reference = TestReference::new();

        let query = b"CGGATGCTCCGGCAGCCGACAGAACGAAGGATCTTGCCGGAAAATGAACTTCTGTTATTATTTTTGTGATTCA";

        let penalties = Penalties {x: 4, o: 5, e: 2};
        let cutoff = Cutoff {
            minimum_aligned_length: 30,
            maximum_penalty_per_scale: 3_000
        };
        let min_penalty_for_pattern = MinPenaltyForPattern { odd: 4, even: 3 };

        let kmer = max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);

        let mut left_wave_front = WaveFront::new_allocated(&penalties, 100);

        let alignment_results = semi_global_alignment_algorithm(
            &mut test_reference,
            query,
            kmer,
            &penalties,
            &cutoff,
            &min_penalty_for_pattern,
            &mut left_wave_front,
        );

        let json = serde_json::to_string(&alignment_results).unwrap();

        println!("{}", json);
    }
}