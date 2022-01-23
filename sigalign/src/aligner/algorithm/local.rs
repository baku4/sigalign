use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
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

// Spare penalty determinant:
// penalty per scale * length - PRECISION_SCALE * penalty
#[derive(Debug)]
struct Anchor {
    query_position: usize,
    record_position: usize,
    size: usize,
    spare_penalty_determinant_of_left: i64,
    left_extension: Option<Extension>,
    right_extension: Option<Extension>,
    dropped: bool,
}


// ALGORITHM


pub fn local_alignment_algorithm<S: SequenceProvider>(
    reference: &Reference<S>,
    query: Sequence,
    pattern_size: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
) -> AlignmentResult {
    let anchors_preset_by_record = Anchors::create_preset_by_record(reference, query, pattern_size);
    let mut sequence_buffer = reference.get_buffer();

    AlignmentResult(
        anchors_preset_by_record.into_iter().filter_map(|(record_index, anchors_preset)| {
            reference.sequence_of_record(record_index, &mut sequence_buffer);
            let record_sequence = sequence_buffer.request_sequence();
            let record_length = record_sequence.len();

            let mut anchors = Anchors::from_preset(anchors_preset, record_length, query, pattern_size, cutoff, min_penalty_for_pattern);
            anchors.extend(record_sequence, query, penalties, cutoff, left_wave_front, right_wave_front);
        
            let alignment_results = anchors.get_alignment_results_for_local();

            if alignment_results.len() == 0 {
                None
            } else {
                Some(
                    RecordAlignmentResult {
                        index: record_index,
                        result: alignment_results,
                    }
                )
            }
        }).collect()
    )
}

// TODO: to del
// #[cfg(test)]
// mod tests {
//     use super::*;

//     use crate::reference::TestReference;

//     fn max_kmer_satisfying_cutoff(cutoff: &Cutoff, min_penalty_for_pattern: &MinPenaltyForPattern) -> usize {
//         let mut n = 1;
//         loop {
//             let upper_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n)  as f32 - 2_f32).ceil();
//             let lower_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n + 2)  as f32 - 2_f32).ceil();
//             let max_penalty = (
//                 (
//                     (
//                         (PRECISION_SCALE * n * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even))
//                     )
//                     + 4 * cutoff.maximum_penalty_per_scale
//                 ) as f32 / (2 * (n+1) * cutoff.maximum_penalty_per_scale) as f32
//             ).ceil() - 2_f32;

//             let kmer = max_penalty.min(upper_bound);
//             #[cfg(test)]
//             println!("#n {}\nu {}\nl {}\nmp {}\nk {}", n, upper_bound, lower_bound, max_penalty, kmer);

//             if kmer >= lower_bound {
//                 return kmer as usize
//             }
//             n += 1;
//         }
//     }

//     #[test]
//     fn print_results_of_local_alignment() {
//         let mut test_reference = TestReference::new();

//         let query = b"CGGATGCTCCGGCAGCCGACAGAACGAAGGATCTTGCCGGAAAATGAACTTCTGTTATTATTTTTGTGATTCA";

//         let penalties = Penalties {x: 4, o: 5, e: 2};
//         let cutoff = Cutoff {
//             minimum_aligned_length: 30,
//             maximum_penalty_per_scale: 3_000,
//         };
//         let min_penalty_for_pattern = MinPenaltyForPattern { odd: 4, even: 3 };

//         let kmer = max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);

//         let mut left_wave_front = WaveFront::new_allocated(&penalties, 100);
//         let mut right_wave_front = WaveFront::new_allocated(&penalties, 100);

//         let alignment_results = local_alignment_algorithm(
//             &mut test_reference,
//             query,
//             kmer,
//             &penalties,
//             &cutoff,
//             &min_penalty_for_pattern,
//             &mut left_wave_front,
//             &mut right_wave_front
//         );

//         println!("alignment_results:\n{:#?}", alignment_results);
//     }
// }