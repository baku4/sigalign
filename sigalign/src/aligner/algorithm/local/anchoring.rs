use super::{PRECISION_SCALE, Cutoff, Penalties, MinPenaltyForPattern};
use super::{ReferenceInterface, Sequence, Reference, SequenceProvider};
use super::{Anchors, Anchor};

mod preset;

use preset::AnchorsPreset;

use std::collections::HashMap;

impl Anchors {
    pub fn create_preset_by_record<'a, S: SequenceProvider<'a>>(
        reference: &Reference<'a, S>,
        query: Sequence,
        pattern_size: usize,
    ) -> HashMap<usize, AnchorsPreset> {
        AnchorsPreset::new_by_record(reference, query, pattern_size)
    }
    pub fn from_preset(
        anchors_preset: AnchorsPreset,
        record_length: usize,
        query: Sequence,
        pattern_size: usize,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) -> Self {
        let anchors = anchors_preset.to_anchors(
            pattern_size,
            query.len(),
            record_length,
            cutoff,
            min_penalty_for_pattern,
        );
        
        anchors
    }
}