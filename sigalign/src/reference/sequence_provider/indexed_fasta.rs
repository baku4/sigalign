use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
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

pub struct IndexedFastaProvider {
    total_record_count: usize,
    line_terminator_size: usize,
    use_reverse_complement: bool,
    fasta_indices: Vec<FastaIndex>,
    // BufReader
    fasta_buf_reader: BufReader<File>,
}

impl IndexedFastaProvider {
    pub fn new<P>(fasta_file_path: P) -> Result<Self> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let mut line_buf_reader = LineBufReader::new(fasta_file_path)?;
        let (fasta_indices, line_terminator_size) = FastaIndex::get_indices_and_line_terminator_size(&mut line_buf_reader)?;

        Ok(
            Self {
                total_record_count: fasta_indices.len(),
                line_terminator_size,
                use_reverse_complement: false,
                fasta_indices,
                fasta_buf_reader: line_buf_reader.buf_reader,
            }
        )
    }
    fn fill_buffer_sequence_from_fasta(&self, record_index: usize, buffer: &mut Vec<u8>) {
        let fasta_index = &self.fasta_indices[record_index];

        let mut new_sequence_buffer = Vec::with_capacity(fasta_index.sequence_length);

        let mut one_line_buffer: Vec<u8> = vec![0; fasta_index.length_of_one_line];

        self.fasta_buf_reader.seek(SeekFrom::Start(fasta_index.sequence_offset)).unwrap();

        // filled line
        for _ in 0..fasta_index.filled_line_count {
            let _ = self.fasta_buf_reader.read_exact(&mut one_line_buffer);
            new_sequence_buffer.extend_from_slice(&one_line_buffer);
            self.fasta_buf_reader.consume(self.line_terminator_size); // TODO: Apply const for better performance
        }

        // last line
        let _ = self.fasta_buf_reader.read_exact(&mut one_line_buffer);
        new_sequence_buffer.extend_from_slice(&one_line_buffer[..fasta_index.length_of_last_line]);

        *buffer = new_sequence_buffer;
    }
}

impl SequenceProvider for IndexedFastaProvider {
    fn total_record_count(&self) -> usize {
        if self.use_reverse_complement {
            self.total_record_count * 2
        } else {
            self.total_record_count
        }
    }
    fn sequence_of_record(&self, record_index: usize, buffer: &mut Vec<u8>) -> Option<&[u8]> {
        if self.use_reverse_complement {
            let record_index_quot = record_index / 2;
            let record_index_rem = record_index % 2;

            self.fill_buffer_sequence_from_fasta(record_index_quot, buffer);

            if record_index_rem == 1 {
                let reverse_complement_sequence = reverse_complement_of_nucleotide_sequence(buffer);
                *buffer = reverse_complement_sequence;
            }
        } else {
            self.fill_buffer_sequence_from_fasta(record_index, buffer);
        }
        None
    }
}

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
