/*!
Implementations for the trait `sigalign_core::reference::SequenceStorage`.

**Current Implementations**
- `in_memory`: Stores the sequences in memory.
  - Generally most fast.
  - Requires enough memory to store all sequences.
*/

pub mod in_memory;