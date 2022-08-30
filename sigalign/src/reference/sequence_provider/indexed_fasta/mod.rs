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


use std::ffi::{OsString, OsStr};
use std::io::{Read, BufRead, BufReader, Seek, SeekFrom, Write};
use std::fs::File;
use std::cell::{Cell, RefCell};
use std::os::unix::prelude::OsStrExt;
use std::sync::{Arc, Mutex};
use std::path::{Path, PathBuf};

use capwriter::{Saveable, Loadable};
use faimm::IndexedFasta as FaiIndexedFasta;

mod reverse_complement;
pub use reverse_complement::IndexedFastaRcProvider;

/// Basic `SequenceProvider` implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedFastaProvider {
    total_record_count: usize,
    fasta_file_path_buf: PathBuf,
}

impl IndexedFastaProvider {
    pub fn new<P>(fasta_file_path: P) -> Result<Self> where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let fasta_file_path_buf = fasta_file_path.as_ref().to_path_buf();

        // TODO: Calculate directly
        let fai_indexed_fasta = FaiIndexedFasta::from_file(&fasta_file_path_buf).unwrap();
        let fai = fai_indexed_fasta.fai();
        let mut last_index = 0;
        while let Ok(_) = fai.size(last_index) {
            last_index += 1;
        };

        Ok(Self {
            total_record_count: last_index,
            fasta_file_path_buf,
        })
    }
    pub fn to_rc_provider(self) -> IndexedFastaRcProvider {
        IndexedFastaRcProvider(self)
    }
}

pub struct IndexedFastaBuffer {
    fai_indexed_fasta: FaiIndexedFasta,
    sequence_buffer: Vec<u8>,
}
impl SequenceBuffer for IndexedFastaBuffer {
    fn request_sequence(&self) -> &[u8] {
        &self.sequence_buffer
    }
}

impl SequenceProvider for IndexedFastaProvider {
    type Buffer = IndexedFastaBuffer;

    fn total_record_count(&self) -> usize {
        self.total_record_count
    }
    fn get_buffer(&self) -> Self::Buffer {
        let fai_indexed_fasta = FaiIndexedFasta::from_file(&self.fasta_file_path_buf).unwrap();

        Self::Buffer {
            fai_indexed_fasta,
            sequence_buffer: Vec::new(),
        }
    }
    fn fill_sequence_buffer(&self, record_index: usize, buffer: &mut Self::Buffer) {
        let fasta_view = buffer.fai_indexed_fasta.view_tid(record_index).unwrap();
        let new_sequence_buffer: Vec<u8> = fasta_view.bases().map(|v| *v).collect();

        buffer.sequence_buffer = new_sequence_buffer;
    }
}

use crate::{EndianType};
use byteorder::{ReadBytesExt, WriteBytesExt};
use bytemuck::{Pod, Zeroable};
use bytemuck::{cast_slice, cast, cast_slice_mut};

// Serializable
impl Serializable for IndexedFastaProvider {
    fn save_to<W>(&self, mut writer: W) -> Result<()> where W: Write {
        // 1. Write total_record_count
        writer.write_u64::<EndianType>(self.total_record_count as u64)?;
        // 2. Write fasta_file_path_buf
        let byte = self.fasta_file_path_buf.as_os_str().as_bytes();
        byte.save_to(writer)?;

        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self> where R: Read, Self: Sized {
        // 1. Read total_record_count
        let total_record_count = reader.read_u64::<EndianType>()? as usize;
        // 2. Read fasta_file_path_buf
        let byte = Vec::<u8>::load_from(reader)?;
        let fasta_file_path_buf = PathBuf::from(OsStr::from_bytes(&byte).to_os_string());

        Ok(Self {
            total_record_count,
            fasta_file_path_buf,
        })
    }
}

// SizeAware
impl SizeAware for IndexedFastaProvider {
    fn size_of(&self) -> usize {
        8 + self.fasta_file_path_buf.as_os_str().as_bytes().size_of()
    }
}
