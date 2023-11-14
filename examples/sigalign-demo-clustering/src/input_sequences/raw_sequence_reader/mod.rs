/*!
Supported input sequences
- One fasta file (PathBuf)
- One fasta bytes (String)
TODO:
- Multiple fasta files (&[PathBuf])
- Multiple sequences (Vec)
*/

mod one_fasta;
pub use one_fasta::OneFastaIterator;
// TODO:
mod multiple_fasta;
mod raw_seq;
pub use raw_seq::RawSequenceIterator;
