pub mod common;


/* Test for `sigalign-utils` crate */
// Sequence reader
mod sequence_reader;


/* Test for `sigalign` crate */
// Validation of SigAlign's result
mod result_validation_only_with_cutoff;
mod result_validation_with_stable_version;
mod results_validation_with_stable_and_dpm;
pub mod result_validation_with_dynamic_programming_matrix;
// Reference acts expectedly
mod reference_gives_correct_data;
mod reference_save_and_load;
// Limited version works
mod limited_version_works;



// Tests for sigalign-impl
// mod implementations;
// Tests for sigalign
// mod wrappers;

// Tests for Documentation
// mod documentation;