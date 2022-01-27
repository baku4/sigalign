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
    sequence_buffer: &mut S::Buffer,
    query: Sequence,
    pattern_size: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
) -> AlignmentResult {
    let anchors_preset_by_record = Anchors::create_preset_by_record(reference, query, pattern_size);

    AlignmentResult(
        anchors_preset_by_record.into_iter().filter_map(|(record_index, anchors_preset)| {
            reference.fill_sequence_buffer(record_index, sequence_buffer);
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
