use super::{
    SequenceStorage, SequenceBuffer,
    ConcatenatedSequenceWithBoundaries,
};
use crate::utils::{path_to_byte, byte_to_pathbuf};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use capwriter::{Saveable, Loadable};
use faimm::IndexedFasta as FaiIndexedFasta;

mod reverse_complement;
pub use reverse_complement::IndexedFastaRcStorage;

/// Basic `SequenceStorage` implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedFastaStorage {
    total_record_count: usize,
    fasta_file_path_buf: PathBuf,
}

impl IndexedFastaStorage {
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
    pub fn to_rc_storage(self) -> IndexedFastaRcStorage {
        IndexedFastaRcStorage(self)
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

impl SequenceStorage for IndexedFastaStorage {
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

// Serializable
impl Serialize for IndexedFastaStorage {
    fn save_to<W>(&self, mut writer: W) -> Result<()> where W: Write {
        // 1. Write total_record_count
        writer.write_u64::<EndianType>(self.total_record_count as u64)?;
        // 2. Write fasta_file_path_buf
        let byte = path_to_byte(&self.fasta_file_path_buf)?;
        byte.save_to(writer)?;

        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self> where R: Read, Self: Sized {
        // 1. Read total_record_count
        let total_record_count = reader.read_u64::<EndianType>()? as usize;
        // 2. Read fasta_file_path_buf
        let byte = Vec::<u8>::load_from(reader)?;
        let fasta_file_path_buf = byte_to_pathbuf(&byte)?;

        Ok(Self {
            total_record_count,
            fasta_file_path_buf,
        })
    }
}

// SizeAware
impl EstimateSize for IndexedFastaStorage {
    fn size_of(&self) -> usize {
        let byte_of_path = self.fasta_file_path_buf.to_str().unwrap().as_bytes();
        8 + byte_of_path.size_of()
    }
}
