use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};
use super::{Extension, AlignmentHashSet, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty_from_determinant};

use std::collections::HashMap;

mod anchoring;
mod extending;

#[derive(Debug, Clone)]
struct ReferenceAnchors {
    anchors_by_record: HashMap<usize, AnchorTable>,
}

type ScaledMargins = Vec<i64>;

impl ReferenceAnchors {
    fn new_for_test() -> (Self, ScaledMargins) {
        (
            Self {
                anchors_by_record: HashMap::new(),
            },
            Vec::new(),
        )
    }
}

#[derive(Debug, Clone)]
struct AnchorTable {
    anchors_by_pattern: Vec<PatternAnchors>,
}

#[derive(Debug, Clone)]
struct PatternAnchors {
    sorted_anchors: Vec<Anchor>,
}

#[derive(Debug, Clone)]
struct Anchor {
    record_position: usize,
    state: AnchorState,
}

#[derive(Debug, Clone)]
enum AnchorState {
    // New
    New,
    // Extended
    RES, // RightExtensionSuccess
    REF, // RightExtensionFail
    RightTraversedSuccess, // 
    RightTraversedFail,
    LeftExtensionSuccess,
    LeftExtensionFail,
    LeftTraversedSuccess,
    LeftTraversedFail,
    // Evaluated
    Fail,
    Included,
    Valid,
    Unique,
}

fn semi_global_alignment_algorithm<S: SequenceProvider>(
    reference: &Reference<S>,
    sequence_buffer: &mut S::Buffer,
    query: Sequence,
    pattern_size: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    wave_front: &mut WaveFront,
) {
    let (reference_anchors, scaled_margins_of_left) = ReferenceAnchors::new_for_test(); // TODO: New

    reference_anchors.anchors_by_record.into_iter().for_each(|(record_index, mut anchor_table)| {
        reference.fill_sequence_buffer(record_index, sequence_buffer);

        let record_sequence = sequence_buffer.request_sequence();
        let record_length = record_sequence.len();

        anchor_table.extend(record_sequence, query, pattern_size, penalties, cutoff, wave_front, &scaled_margins_of_left)
    });
}