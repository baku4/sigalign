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
};

mod indexing;

use crate::util::reverse_complement_of_nucleotide_sequence;
use std::io::{Read, BufRead, BufReader, Seek, SeekFrom, Write};
use std::fs::File;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};
use std::path::Path;
use serde::{Serialize, Deserialize};
use bincode::{serialize_into, deserialize_from};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct IndexedFastaProvider {
    total_record_count: usize,
    line_terminator_size: usize,
    use_reverse_complement: bool,
    fasta_indices: Vec<FastaIndex>,
    fasta_file_path: String,
}

impl IndexedFastaProvider {
    pub fn new<P>(fasta_file_path: P) -> Result<Self> where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let string_fasta_file_path = match fasta_file_path.as_ref().to_str() {
            Some(v) => v.to_string(),
            None => error_msg!("Invalid fasta file path")
        };

        let mut line_buf_reader = LineBufReader::new(fasta_file_path)?;
        let (fasta_indices, line_terminator_size) = FastaIndex::get_indices_and_line_terminator_size(&mut line_buf_reader)?;

        Ok(
            Self {
                total_record_count: fasta_indices.len(),
                line_terminator_size,
                use_reverse_complement: false,
                fasta_indices,
                fasta_file_path: string_fasta_file_path,
            }
        )
    }
    fn fill_buffer_sequence_from_fasta(&self, record_index: usize, buffer: &mut IndexedFastaBuffer) {
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

impl SequenceProvider for IndexedFastaProvider {
    type Buffer = IndexedFastaBuffer;

    fn total_record_count(&self) -> usize {
        if self.use_reverse_complement {
            self.total_record_count * 2
        } else {
            self.total_record_count
        }
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
        if self.use_reverse_complement {
            let record_index_quot = record_index / 2;
            let record_index_rem = record_index % 2;

            self.fill_buffer_sequence_from_fasta(record_index_quot, buffer);

            if record_index_rem == 1 {
                let reverse_complement_sequence = reverse_complement_of_nucleotide_sequence(&buffer.sequence_buffer);
                buffer.sequence_buffer = reverse_complement_sequence;
            }
        } else {
            self.fill_buffer_sequence_from_fasta(record_index, buffer);
        }
    }
}

// Label Provider
impl LabelProvider for IndexedFastaProvider {
    fn label_of_record(&self, record_index: usize) -> String {
        if self.use_reverse_complement {
            let record_index_quot = record_index / 2;
            let fasta_index = &self.fasta_indices[record_index_quot];
            fasta_index.label.clone()
        } else {
            let fasta_index = &self.fasta_indices[record_index];
            fasta_index.label.clone()
        }
    }
}

// Serializable
impl Serializable for IndexedFastaProvider {
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

struct IndexedFastaBuffer {
    fasta_buf_reader: BufReader<File>,
    sequence_buffer: Vec<u8>,
}

impl SequenceBuffer for IndexedFastaBuffer {
    fn request_sequence(&self) -> &[u8] {
        &self.sequence_buffer
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FastaIndex {
    label: String,
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
