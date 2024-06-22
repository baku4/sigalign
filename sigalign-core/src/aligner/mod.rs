/*!
Alignment executors.
*/

// Common components
//  - To make workspace
mod workspace;
//  - To define input parameters
mod regulator;
pub use regulator::RegulatorError;

// Executing local alignment algorithm.
pub mod local;
// Executing semi-global alignment algorithm.
pub mod semi_global;
