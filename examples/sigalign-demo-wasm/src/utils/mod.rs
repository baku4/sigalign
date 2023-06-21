use super::{Reference, Aligner};

mod error_handling;
mod sequence_sampling;

pub(crate) use error_handling::err_to_js_err;
pub use sequence_sampling::{
    get_sample_target_as_fasta_string,
    get_sample_query,
};
