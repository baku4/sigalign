use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference, SequenceProvider, JoinedSequence,
    SequenceType, PatternFinder,
    // Trait
    Serializable, SizeAware,
    LabelProvider,
    ReverseComplement,
};
use super::{
    IndexedFastaProvider,
    IndexedFastaBuffer,
    FaiIndexedFasta,
};

use std::ffi::{OsString, OsStr};
use std::io::{Read, BufRead, BufReader, Seek, SeekFrom, Write};
use std::fs::File;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};

use crate::util::reverse_complement_of_nucleotide_sequence;

pub struct IndexedFastaRcProvider(pub(super) IndexedFastaProvider);

impl IndexedFastaRcProvider {
    pub fn new<P>(fasta_file_path: P) -> Result<Self> where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let indexed_fasta_provider = IndexedFastaProvider::new(fasta_file_path)?;
        
        Ok(Self(indexed_fasta_provider))
    }
    pub fn to_non_rc_provider(self) -> IndexedFastaProvider {
        self.0
    }
}

impl SequenceProvider for IndexedFastaRcProvider {
    type Buffer = IndexedFastaBuffer;

    fn total_record_count(&self) -> usize {
        self.0.total_record_count * 2
    }
    fn get_buffer(&self) -> Self::Buffer {
        self.0.get_buffer()
    }
    fn fill_sequence_buffer(&self, record_index: usize, buffer: &mut Self::Buffer) {
        let record_index_quot = record_index / 2;
        let record_index_rem = record_index % 2;

        self.0.fill_sequence_buffer(record_index_quot, buffer);

        if record_index_rem == 1 {
            let reverse_complement_sequence = reverse_complement_of_nucleotide_sequence(&buffer.sequence_buffer);
            buffer.sequence_buffer = reverse_complement_sequence;
        }
    }
}

// Reverse Complement
impl ReverseComplement for IndexedFastaRcProvider {
    fn is_reverse_complement(&self, record_index: usize) -> bool {
        if record_index % 2 == 0 {
            false
        } else {
            true
        }
    }
}

// Serializable
impl Serializable for IndexedFastaRcProvider {
    fn save_to<W>(&self, writer: W) -> Result<()> where W: Write {
        self.0.save_to(writer)
    }
    fn load_from<R>(reader: R) -> Result<Self> where R: Read, Self: Sized {
        let indexed_fasta_provider = IndexedFastaProvider::load_from(reader)?;
        Ok(Self(indexed_fasta_provider))
    }
}

// SizeAware
impl SizeAware for IndexedFastaRcProvider {
    fn size_of(&self) -> usize {
        self.0.size_of()
    }
}
