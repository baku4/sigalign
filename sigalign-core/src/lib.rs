/*!
The collection of the most **fundamental components** to execute the SigAlign algorithm.

The purpose of `sigalign-core`` is to hide the complexity of the detailed implementation of the SigAlign algorithm.

`sigalign-core` allows for lower-level control compared to `sigalign` crate. It can be used with `sigalign-impl` and `sigalign-utils` to build a custom alignment tool. However, it does not have user-friendly interfaces implemented in `sigalign`. 
*/

mod core;
mod algorithm;

pub mod results;
pub mod reference;
pub mod aligner;
