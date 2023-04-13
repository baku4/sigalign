use super::{
    Anchor,
    AnchorIndex,
    AnchorTable,
};
use ahash::AHashSet;

pub const LEFT_EMPTY_SYMBOL: i32 = -1;
pub const RIGHT_EMPTY_SYMBOL: i32 = -2;
pub const MERGED_EMPTY_SYMBOL: i32 = -3;