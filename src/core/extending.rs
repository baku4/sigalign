use super::Penalties;
use super::Sequence;

mod dwfa;

pub use dwfa::{DropoffWaveFront, WaveFrontScore, Components, Component};
pub use dwfa::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};