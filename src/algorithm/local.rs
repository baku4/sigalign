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
    spare_penalty_padding_of_left: f32,
    left_extension: Option<Extension>,
    right_extension: Option<Extension>,
    dropped: bool,
}

#[derive(Debug, Clone)]
pub struct Extension {
    penalty: usize,
    length: usize,
    insertion_count: u32,
    deletion_count: u32,
    operations: Vec<AlignmentOperation>,
}


// ALGORITHM


pub fn local_alignment(
    reference: &dyn Reference,
    query: Sequence,
    pattern_size: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
) {
    let anchors_preset_by_record = Anchors::create_preset_by_record(reference, query, pattern_size);

    anchors_preset_by_record.into_iter().filter_map(|(record_index, anchors_preset)| {
        let record_sequence = reference.sequence_of_record(record_index);
        let record_length = record_sequence.len();

        let mut anchors = Anchors::from_preset(anchors_preset, record_length, query, pattern_size, cutoff, min_penalty_for_pattern);

        anchors.extend(record_sequence, query, penalties, cutoff);
       
        let alignment_results = anchors.get_alignment_results_for_local();

        if alignment_results.len() == 0 {
            None
        } else {
            Some((record_index, alignment_results))
        }
    });
}