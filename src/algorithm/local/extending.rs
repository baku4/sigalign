use super::{Cutoff, Penalties};
use super::{Sequence};
use super::{AlignmentOperation, AlignmentType};
use super::{Anchors, Anchor, Extension};
use super::{DropoffWaveFront, WaveFrontScore, Components, Component};
use super::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};

mod dwfa;

impl Anchors {
    pub fn extend(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        #[cfg(test)]
        println!("# extend right");
        self.extend_right(record_sequence, query, penalties, cutoff);
        #[cfg(test)]
        println!("{:#?}", self);
        // #[cfg(test)]
        // println!("# extend left");
        self.extend_left(record_sequence, query, penalties, cutoff);
        // #[cfg(test)]
        // println!("{:#?}", self);
    }
    fn extend_right(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        self.anchors.iter_mut().for_each(|anchor| {
            
        })
    }
    fn extend_left(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        
    }
}