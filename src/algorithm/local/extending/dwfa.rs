use super::{Cutoff, Penalties};
use super::{Sequence};
use super::{AlignmentOperation, AlignmentType};
use super::{Anchors, Anchor, Extension};
use super::{DropoffWaveFront, WaveFrontScore, Components, Component};
use super::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};

impl DropoffWaveFront<ComponentLocal> {
    pub fn align_right_for_local(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
    ) -> Option<Extension> {
        let dropoff_wave_front = Self::aligned_forward(ref_seq, qry_seq, penalties, spare_penalty);

        
    }
    pub fn align_left_for_local(
        ref_seq: Sequence,
        qry_seq: Sequence,
        penalties: &Penalties,
        spare_penalty: usize,
    ) -> Option<Extension> {
        let dropoff_wave_front = Self::aligned_reverse(ref_seq, qry_seq, penalties, spare_penalty);


    }
}

struct ComponentLocal {
    fr: i32,
    deletion_count: u16,
    bt: u8,
}

impl Component for ComponentLocal {

}