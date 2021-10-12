use super::{Penalties, Cutoff, MinPenaltyForPattern};
use super::{Sequence, Reference, PatternLocation};
use super::{AlignmentResultsByRecord, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};
use super::{DropoffWaveFront, WaveFrontScore, Components, Component};
use super::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};

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


pub fn semi_global_alignment(
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

        let alignment_results = anchors.get_alignment_result_for_semi_global(cutoff);

        if alignment_results.len() == 0 {
            None
        } else {
            Some((record_index, alignment_results))
        }
    }).collect()
}
