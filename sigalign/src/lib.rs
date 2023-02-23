// #[cfg(target_endian = "little")]
// type EndianType = byteorder::LittleEndian;
// #[cfg(target_endian = "big")]
// type EndianType = byteorder::BigEndian;

#[doc(hidden)]
// Core
//  - Widely used structures and policy
//  - For advanced users
pub mod core;

// Implementations
pub mod reference;
pub mod aligner;
/// wrapper
pub mod wrapper;

/// Utilities
pub mod utils;
/// Collection of error
pub mod errors;

// Structure of alignment result
// pub mod result;
// (4) Public APIs
// mod alignment; // Alignment functions
// mod builder; // Struct builders
