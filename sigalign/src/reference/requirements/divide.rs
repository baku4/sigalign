pub struct DivisionError;

/// Divide into several chunks
pub trait Divide {
    // Split sequence storage to specific max sized length.
    // If one record exceeds the max length, splitted storage can contain only one of that record.

    /// Divide sequence storage to specific max sized length.
    fn divide_into(self, count: usize) -> Result<Vec<Self>, DivisionError> where Self: Sized;
}
