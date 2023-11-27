pub use anyhow::{Result, bail as error_msg};

// Logger
mod logger;
pub use logger::init_logger;

// Test Data
pub mod test_data_path;
pub mod random_text_and_pattern;

// Result of stable version of sigalign
mod result_of_stable_version;
pub use result_of_stable_version::{
    get_semi_global_result_of_stable_version,
    get_local_result_of_stable_version,
};

// DP matrix to generate the answer result
// pub mod dynamic_programming_matrix;