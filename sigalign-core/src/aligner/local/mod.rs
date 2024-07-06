use super::regulator::AlignmentRegulator;

mod workspace;
use workspace::LocalWorkspace;

mod local_unlimited;
pub use local_unlimited::LocalAligner;
mod local_with_limit;
pub use local_with_limit::LocalWithLimitAligner;

mod switch_modes;
