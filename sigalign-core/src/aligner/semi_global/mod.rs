use super::regulator::AlignmentRegulator;

mod workspace;
use workspace::SemiGlobalWorkspace;

mod semi_global_unlimited;
pub use semi_global_unlimited::SemiGlobalAligner;
mod semi_global_with_limit;
pub use semi_global_with_limit::SemiGlobalWithLimitAligner;

mod switch_modes;