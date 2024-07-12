use super::{DpMatrix, Cell, BacktraceMarker};

mod common;
use common::{
    parse_the_unoverlapped_alignments_with_path,
    parse_the_unique_alignments_and_its_path,
};
// Criteria to print output is changed.
// Old version:
//   - print the alignment whose position is not overlapped with "the existing results".
// New version:
//   - print the alignment whose position is overlapped with "the more optimal results".
mod semi_global;
pub use semi_global::parse_valid_semi_global_result_from_dpm;
mod local;
pub use local::parse_valid_local_result_from_dpm;

mod local_old;
pub use local_old::parse_valid_local_result_old;
mod semi_global_old;
pub use semi_global_old::parse_valid_semi_global_result_old;


