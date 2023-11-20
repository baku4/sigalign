// Common data structures and functions
mod anchor;
use anchor::{
    Anchor,
    AnchorTable,
};
pub use anchor::AnchorIndex;
use anchor::mark_traversed_anchors_as_skipped;

mod wave_front;
use wave_front::{
    WaveFrontScore,
    BackTraceMarker,
    BackTraceResult,
};
pub use wave_front::WaveFront;

mod spare_penalty;
pub use spare_penalty::SparePenaltyCalculator;

mod traversed;
use traversed::{
    transform_left_additive_position_to_traversed_anchor_index,
    transform_right_additive_position_to_traversed_anchor_index,
};

mod extension;
pub use extension::Extension;

// Alignment algorithms
mod semi_global;
pub use semi_global::{
    semi_global_alignment_algorithm,
    semi_global_alignment_algorithm_with_limit,
};

mod local;
pub use local::{
    local_alignment_algorithm,
    local_alignment_algorithm_with_limit,
    Vpc,
};
