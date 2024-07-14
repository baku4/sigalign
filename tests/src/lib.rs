pub mod common;


/* Test for `sigalign-utils` crate */
// Sequence reader
mod sequence_reader;


/* Test for `sigalign` crate */
// Result validation with stable version
mod result_validation_only_with_cutoff;
mod result_validation_with_stable_version;
mod debug_alignment_results;
pub mod result_validation_with_dynamic_programming_matrix;


// Tests for sigalign-impl
mod implementations;
// Tests for sigalign
// mod wrappers;

// Tests for Documentation
// mod documentation;