// Common steps for local and semi-global algorithms

// 1. Make position table
mod pos_table;
pub use pos_table::{PosTable, AnchorIndex, AnchorPosition};

// 2. Calculate spare penalty
mod spare_penalty;
pub use spare_penalty::{calculate_spare_penalty};

// // 3. Extend the anchors
mod extend;
pub use extend::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

// // 4. Backtrace from the extension
mod backtrace;
pub use backtrace::{TraversedPosition, TraversedAnchorDep};

// // 5. Merge the extension
mod merging;
