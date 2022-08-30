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

mod indexing;
mod reverse_complement;
pub use reverse_complement::IndexedFastaRcProvider;

use std::io::{Read, BufRead, BufReader, Seek, SeekFrom, Write};
use std::fs::File;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use std::path::Path;

use capwriter::{Saveable, Loadable};
use serde::{Serialize, Deserialize};

/// Basic `SequenceProvider` implementation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexedFastaProvider {
    total_record_count: usize,
    line_terminator_size: usize,
    fasta_indices: Vec<FastaIndex>,
    fasta_file_path: String,
}

impl IndexedFastaProvider {
    pub fn new<P>(fasta_file_path: P) -> Result<Self> where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let string_fasta_file_path = match fasta_file_path.as_ref().canonicalize()?.to_str() {
            Some(v) => v.to_string(),
            None => error_msg!("Invalid fasta file path")
        };

        let mut line_buf_reader = LineBufReader::new(fasta_file_path)?;
        let (fasta_indices, line_terminator_size) = FastaIndex::get_indices_and_line_terminator_size(&mut line_buf_reader)?;

        Ok(
            Self {
                total_record_count: fasta_indices.len(),
                line_terminator_size,
                fasta_indices,
                fasta_file_path: string_fasta_file_path,
            }
        )
    }
    pub fn to_rc_provider(self) -> IndexedFastaRcProvider {
        IndexedFastaRcProvider(self)
    }
}

impl SequenceProvider for IndexedFastaProvider {
    type Buffer = IndexedFastaBuffer;

    fn total_record_count(&self) -> usize {
        self.total_record_count
    }
    fn get_buffer(&self) -> Self::Buffer {
        let file = File::open(&self.fasta_file_path).unwrap();
        let buf_reader = BufReader::new(file);

        Self::Buffer {
            fasta_buf_reader: buf_reader,
            sequence_buffer: Vec::new(),
        }
    }
    fn fill_sequence_buffer(&self, record_index: usize, buffer: &mut Self::Buffer) {
        let fasta_index = &self.fasta_indices[record_index];

        let mut new_sequence_buffer = Vec::with_capacity(fasta_index.sequence_length);

        let mut one_line_buffer: Vec<u8> = vec![0; fasta_index.length_of_one_line];

        buffer.fasta_buf_reader.seek(SeekFrom::Start(fasta_index.sequence_offset)).unwrap();

        // filled line
        for _ in 0..fasta_index.filled_line_count {
            let _ = buffer.fasta_buf_reader.read_exact(&mut one_line_buffer);
            new_sequence_buffer.extend_from_slice(&one_line_buffer);
            buffer.fasta_buf_reader.consume(self.line_terminator_size); // TODO: Apply const for better performance
        }

        // last line
        let _ = buffer.fasta_buf_reader.read_exact(&mut one_line_buffer);
        new_sequence_buffer.extend_from_slice(&one_line_buffer[..fasta_index.length_of_last_line]);

        buffer.sequence_buffer = new_sequence_buffer;
    }
}

use crate::{EndianType};
use byteorder::{ReadBytesExt, WriteBytesExt};
use bytemuck::{Pod, Zeroable};
use bytemuck::{cast_slice, cast, cast_slice_mut};

// Serializable
impl Serializable for IndexedFastaProvider {
    fn save_to<W>(&self, mut writer: W) -> Result<()> where
        W: std::io::Write
    {
        // 1. Write total_record_count
        writer.write_u64::<EndianType>(self.total_record_count as u64)?;
        // 2. Write line_terminator_size
        writer.write_u64::<EndianType>(self.line_terminator_size as u64)?;
        // 3. Write fasta_indices
        //  length
        writer.write_u64::<EndianType>(self.fasta_indices.len() as u64)?;
        //  slice
        let slice: &[u8] = cast_slice(&self.fasta_indices);
        writer.write_all(slice)?;
        // 4. Write fasta_file_path
        let slice: &[u8] = self.fasta_file_path.as_bytes();
        slice.save_to(writer)?;
        Ok(())
    }
    fn load_from<R>(mut reader: R) -> Result<Self> where
        R: std::io::Read,
        Self: Sized,
    {
        // 1. Read total_record_count
        let total_record_count = reader.read_u64::<EndianType>()? as usize;
        // 2. Read line_terminator_size
        let line_terminator_size = reader.read_u64::<EndianType>()? as usize;
        // 3. Read fasta_indices
        let len = reader.read_u64::<EndianType>()? as usize;
        let mut fasta_indices: Vec<FastaIndex> = vec![FastaIndex::zeroed(); len];
        let buffer: &mut [u8] = cast_slice_mut(&mut fasta_indices);
        reader.read_exact(buffer)?;
        // 4. Read fasta_file_path
        let byte = Vec::<u8>::load_from(reader)?;
        let fasta_file_path = String::from_utf8(byte)?;
        
        Ok(Self {
            total_record_count,
            line_terminator_size,
            fasta_indices,
            fasta_file_path,
        })
    }
}

// Size aware
impl SizeAware for IndexedFastaProvider {
    fn size_of(&self) -> usize {
        8 // total_record_count
        + 8 // line_terminator_size
        + 8 + (self.fasta_indices.len() * std::mem::size_of::<FastaIndex>() as usize) // fasta_indices
        + self.fasta_file_path.as_bytes().len() // fasta_file_path
    }
}

pub struct IndexedFastaBuffer {
    fasta_buf_reader: BufReader<File>,
    sequence_buffer: Vec<u8>,
}

impl SequenceBuffer for IndexedFastaBuffer {
    fn request_sequence(&self) -> &[u8] {
        &self.sequence_buffer
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Pod, Zeroable)]
struct FastaIndex {
    sequence_offset: u64,
    sequence_length: usize,
    length_of_one_line: usize,
    filled_line_count: usize,
    length_of_last_line: usize,
    // EXAMPLE
    // --
    // > desc
    // TTGGTGCGAG
    // CTTCTCTCTG
    // TCCGCATA
    // 
    // =>
    // length_of_one_line: 10,
    // filled_line_count: 2,
    // length_of_last_line: 8,
}

struct LineBufReader {
    buf_reader: BufReader<File>,
    buffer: String,
}

impl LineBufReader {
    fn new<P>(file_path: P) -> Result<Self> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let file = File::open(file_path)?;
        Ok(Self {
            buf_reader: BufReader::new(file),
            buffer: String::new(),
        })
    }
}

impl Iterator for LineBufReader {
    type Item = String;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.clear();
        self.buf_reader.read_line(&mut self.buffer).unwrap();
        if self.buffer.len() == 0 {
            None
        } else {
            Some(self.buffer.clone())
        }
    }
}
