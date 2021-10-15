use super::{Penalties, Cutoff, MinPenaltyForPattern};
use super::{Sequence, Reference, PatternLocation};
use super::{AlignmentResultsByRecord, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType, AlignmentHashSet};
use super::{DropoffWaveFront, WaveFrontScore, Components, Component};
use super::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};
use super::Algorithm;

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
    left_extension: Option<Extension>,
    right_extension: Option<Extension>,
    dropped: bool,
    connected_anchors: Vec<usize>,
}

#[derive(Debug)]
struct Estimation {
    penalty: usize,
    length: usize,
}

#[derive(Debug, Clone)]
pub struct Extension {
    penalty: usize,
    length: usize,
    insertion_count: u32,
    deletion_count: u32,
    operations: OperationsOfExtension,
}

#[derive(Debug, Clone)]
enum OperationsOfExtension {
    Own(OwnedOperations),
    Ref(RefToOperations),
}

#[derive(Debug, Clone)]
struct OwnedOperations {
    operations: Vec<AlignmentOperation>,
}

#[derive(Debug, Clone)]
struct RefToOperations {
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


pub struct SemiGlobalAlgorithm;

impl Algorithm for SemiGlobalAlgorithm {
    fn alignment(
        reference: &dyn Reference,
        query: Sequence,
        pattern_size: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) -> AlignmentResultsByRecord {
        let anchors_preset_by_record = Anchors::create_preset_by_record(reference, query, pattern_size);

        anchors_preset_by_record.into_iter().filter_map(|(record_index, anchors_preset)| {
            let record_sequence = reference.sequence_of_record(record_index);
            let record_length = record_sequence.len();

            let mut anchors = Anchors::from_preset(anchors_preset, record_length, query, pattern_size, cutoff, penalties, min_penalty_for_pattern);

            anchors.extend(record_sequence, query, penalties, cutoff);

            let alignment_results = anchors.get_alignment_results_for_semi_global(cutoff);

            if alignment_results.len() == 0 {
                None
            } else {
                Some((record_index, alignment_results))
            }
        }).collect()
    }
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
                ((n*(min_penalty_for_pattern.odd + min_penalty_for_pattern.even)) as f32
                + 4_f32*cutoff.penalty_per_length) /
                (2_f32*cutoff.penalty_per_length*(n+1) as f32)
                - 2_f32
            ).ceil();

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
        let test_reference = TestReference::new();

        let query = b"CGGATGCTCCGGCAGCCGACAGAACGAAGGATCTTGCCGGAAAATGAACTTCTGTTATTATTTTTGTGATTCA";

        let penalties = Penalties {x: 4, o: 5, e: 2};
        let cutoff = Cutoff { minimum_aligned_length: 30, penalty_per_length: 0.3 };
        let min_penalty_for_pattern = MinPenaltyForPattern { odd: 4, even: 3 };

        let kmer = max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);

        let alignment_results = SemiGlobalAlgorithm::alignment(
            &test_reference,
            query,
            kmer,
            &penalties,
            &cutoff,
            &min_penalty_for_pattern
        );

        println!("alignment_results:\n{:#?}", alignment_results);
    }

    #[test]
    fn print_json_results_of_semi_global_alignment() {
        let test_reference = TestReference::new();

        let query = b"CGGATGCTCCGGCAGCCGACAGAACGAAGGATCTTGCCGGAAAATGAACTTCTGTTATTATTTTTGTGATTCA";

        let penalties = Penalties {x: 4, o: 5, e: 2};
        let cutoff = Cutoff { minimum_aligned_length: 30, penalty_per_length: 0.3 };
        let min_penalty_for_pattern = MinPenaltyForPattern { odd: 4, even: 3 };

        let kmer = max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);

        let alignment_results = SemiGlobalAlgorithm::alignment(
            &test_reference,
            query,
            kmer,
            &penalties,
            &cutoff,
            &min_penalty_for_pattern
        );

        let json = serde_json::to_string(&alignment_results).unwrap();

        println!("{}", json);
    }
}