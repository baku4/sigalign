use super::{
    Result,
};

/// Divide into several chunks
pub trait Divide {
    // Split sequence storage to specific max sized length.
    // If one record exceeds the max length, splitted storage can contain only one of that record.
    fn split_by_max_length(self, max_length: usize) -> Result<Vec<Self>> where Self: Sized;
}
