// Scenarios:
// - (1) Generate the test results with DPM in server
// - (2) Download cached results
// - (3) When running the tests, compare the results with the cached results.

mod dpm_test_unit;
pub use dpm_test_unit::{
    DpmTestUnit, DpmTestMode,
};

mod generate_test_results_with_dpm;
