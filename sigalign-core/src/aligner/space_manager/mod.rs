use super::AlignmentRegulator;

mod wave_front_pool;
use wave_front_pool::{
    WaveFrontPool,
    SingleWaveFrontPool,
    DoubleWaveFrontPool,
};

mod query_length_checker;
use query_length_checker::QueryLengthChecker;
pub use query_length_checker::AllocationStrategy;

mod single_regulator;
pub use single_regulator::{
    SingleSpaceManager,
    SingleLocalSpaceManager,
    SingleSemiGlobalSpaceManager,
};
mod multiple_regulators;
pub use multiple_regulators::{
    MultipleSpaceManager,
    MultipleLocalSpaceManager,
    MultipleSemiGlobalSpaceManager,
};
