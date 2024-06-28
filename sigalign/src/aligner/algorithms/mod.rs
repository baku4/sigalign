use super::{
    Reference, DefaultSequenceBuffer,
    QueryAlignment,
};

mod error;
pub use error::ParamsError;
use error::check_pattern_size;

mod basic;
mod with_limit;
// mod with_chunk; TODO: WORKING ON
pub use basic::{Local, SemiGlobal};
pub use with_limit::{LocalWithLimit, SemiGlobalWithLimit};

pub trait Algorithm {
    fn align(
        &mut self,
        query: &[u8],
        reference: &Reference,
        sequence_buffer: &mut DefaultSequenceBuffer,
    ) -> QueryAlignment;
}
