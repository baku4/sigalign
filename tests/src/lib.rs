pub mod common;


/* Test for `sigalign-utils` crate */
// Sequence reader
mod sequence_reader;


/* Test for `sigalign` crate */
// Validation of SigAlign's result
mod results_satisfy_cutoff;
mod limitation_of_results_works;
mod results_validation_with_032_and_dpm;
pub mod result_validation_with_dynamic_programming_matrix;
// Reference acts expectedly
mod reference_gives_correct_data;
mod reference_save_and_load;


// Tests for Documentation
mod documentation;