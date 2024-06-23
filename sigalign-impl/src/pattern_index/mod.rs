/*!
Implementations for the trait `sigalign_core::reference::PatternIndex`.

**Current Implementations**
- Using `LtFmIndex` (<https://github.com/baku4/lt-fm-index>):
  - `static_lfi`: Has a maximum number of characters that can be indexed.
  - `dynamic_lfi`: Can adjust the internal type by the number of characters (slightly slower than static version).
*/
pub mod static_lfi;
pub mod dynamic_lfi;
