/*!
Supplementary functionalities for the `Reference`.

This module houses auxiliary features that can be utilized by `Reference` given the proper implementation by `SequenceStorage` or `PatternIndex`. For instance, if both `SequenceStorage` and `PatternIndex` support the [Serialize] trait, a `Reference` defined by these implementations can benefit from serialization and deserialization.
*/
use super::{
    Reference,
    PatternIndex,
    SequenceStorage,
};

mod io;
pub use io::{
    Serialize,
    EstimateSize,
};
mod label;
pub use label::LabelStorage;
mod clone;
