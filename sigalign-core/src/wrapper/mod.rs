/*!
Easy to use wrappers.
*/
mod reference;
pub use reference::DefaultReference;

mod aligner;
pub use aligner::{
    DefaultAligner,
    ModeSwitchError,
};
