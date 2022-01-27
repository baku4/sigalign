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
    Serializable,
    LabelProvider,
    ReverseComplement,
};
use super::{IndexedFastaProvider, IndexedFastaBuffer};

use crate::util::{FastaReader, reverse_complement_of_nucleotide_sequence};

use std::io::{Read, BufRead, BufReader, Seek, SeekFrom, Write};
use std::fs::File;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use std::path::Path;
use serde::{Serialize, Deserialize};
use bincode::{serialize_into, deserialize_from};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedFastaRcProvider(
    pub(super) IndexedFastaProvider
);

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

// Label Provider
impl LabelProvider for IndexedFastaRcProvider {
    fn label_of_record(&self, record_index: usize) -> String {
        let record_index_quot = record_index / 2;
        self.0.label_of_record(record_index_quot)
    }
}

// Serializable
impl Serializable for IndexedFastaRcProvider {
    fn save_to<W>(&self, writer: W) -> Result<()> where
        W: std::io::Write
    {
        serialize_into(writer, self)?;
        Ok(())
    }
    fn load_from<R>(reader: R) -> Result<Self> where
        R: std::io::Read,
        Self: Sized,
    {
        let value: Self = deserialize_from(reader)?;
        Ok(value)
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
