use crate::{Result, error_msg};
use crate::{Serialize, Deserialize};
use super::{SequenceProvider, Labeling, FastaReader, reverse_complement_of_nucleotide_sequence};

use std::path::Path;
use std::fs::File;
use std::io::{Read, BufRead, BufReader, Seek, SeekFrom};
use std::ffi::OsString;

use serde::ser::{Serializer, SerializeStruct};

const LF_TERMINATION_SIZE: usize = 1;
const CRLF_TERMINATION_SIZE: usize = 2;

pub struct IndexedFastaProvider {
    total_record_count: usize,
    line_terminator_size: usize,
    use_reverse_complement: bool,
    fasta_indices: Vec<FastaIndex>,
    // Fasta information
    fasta_file_path: OsString,
    fasta_buf_reader: BufReader<File>,
    // Buffers
    sequence_buffer: Vec<u8>,
}
impl SequenceProvider for IndexedFastaProvider {
    fn total_record_count(&self) -> usize {
        if self.use_reverse_complement {
            self.total_record_count * 2
        } else {
            self.total_record_count
        }
    }
    fn sequence_of_record(&mut self, record_index: usize) -> &[u8] {
        if self.use_reverse_complement {
            let record_index_quot = record_index / 2;
            let record_index_rem = record_index % 2;

            self.fill_buffer_sequence_from_fasta(record_index_quot);

            if record_index_rem == 1 {
                let reverse_complement_sequence = reverse_complement_of_nucleotide_sequence(&self.sequence_buffer);
                self.sequence_buffer = reverse_complement_sequence;
            }
        } else {
            self.fill_buffer_sequence_from_fasta(record_index);
        } 
        
        &self.sequence_buffer
    }
}
impl Labeling for IndexedFastaProvider {
    fn label_of_record(&mut self, record_index: usize) -> &str {
        &self.fasta_indices[record_index].label
    }
}
impl IndexedFastaProvider {
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(
        fasta_file_path: P,
    ) -> Result<Self> {
        let fasta_file_path = fasta_file_path.as_ref().as_os_str().to_os_string();

        let mut fasta_indices = Vec::new();

        let mut fasta_buf_reader = FastaBufReader::new(fasta_file_path.clone())?;
        let mut offset_to_current_line = 0;
        let mut offset_to_sequence_start_point = 0;
        let mut sequence_length = 0;
        let mut sequence_length_of_lines = Vec::new();
        let mut label = String::new();

        // Init with first line (This line is always description)
        let line_terminator_size = if let Some(line) = fasta_buf_reader.next() {
            // add offset
            let offset = line.len();
            offset_to_current_line += offset; 
            offset_to_sequence_start_point = offset_to_current_line;

            let line_terminator = if line.ends_with("\r\n") {
                CRLF_TERMINATION_SIZE
            } else if line.ends_with("\n") {
                LF_TERMINATION_SIZE
            } else {
                error_msg!("Line terminator cannot defined in fasta file.");
            };

            label = parse_label_from_line(line);

            line_terminator
        } else {
            error_msg!("No record in fasta file.");
        };

        while let Some(line) = fasta_buf_reader.next() {
            if line.starts_with(">") { // line is description
                // Push new FastaIndex
                let new_fasta_index = if sequence_length_of_lines.len() == 1 {
                    FastaIndex {
                        label,
                        sequence_offset: offset_to_sequence_start_point as u64,
                        sequence_length: sequence_length,
                        length_of_one_line: sequence_length_of_lines[0],
                        filled_line_count: 0,
                        length_of_last_line: sequence_length_of_lines[0],
                    }
                } else {
                    FastaIndex {
                        label,
                        sequence_offset: offset_to_sequence_start_point as u64,
                        sequence_length: sequence_length,
                        length_of_one_line: sequence_length_of_lines[0],
                        filled_line_count: sequence_length_of_lines.len() - 1,
                        length_of_last_line: sequence_length_of_lines[sequence_length_of_lines.len() - 1],
                    }
                };
                fasta_indices.push(new_fasta_index);

                // Deal with this line
                // add offset
                let offset = line.len();
                offset_to_current_line += offset;
                offset_to_sequence_start_point = offset_to_current_line;
                
                label = parse_label_from_line(line);
                sequence_length = 0;
                sequence_length_of_lines.clear();
            } else { // Line is of sequence
                let offset = line.len();
                offset_to_current_line += offset;
                let sequence_length_of_this_line = offset - line_terminator_size;
                sequence_length += sequence_length_of_this_line;
                sequence_length_of_lines.push(sequence_length_of_this_line);
            }
        }

        Ok(
            Self {
                total_record_count: fasta_indices.len(),
                line_terminator_size: line_terminator_size,
                use_reverse_complement: false,
                fasta_indices: fasta_indices,
                fasta_file_path: fasta_file_path,
                fasta_buf_reader: fasta_buf_reader.buf_reader,
                sequence_buffer: Vec::new(),
            }
        )        
    }
    fn fill_buffer_sequence_from_fasta(
        &mut self,
        record_index: usize,
    ) {
        let fasta_index = &self.fasta_indices[record_index];

        self.sequence_buffer = Vec::with_capacity(fasta_index.sequence_length);

        let mut one_line_buffer: Vec<u8> = vec![0; fasta_index.length_of_one_line];

        self.fasta_buf_reader.seek(SeekFrom::Start(fasta_index.sequence_offset)).unwrap();

        // filled line
        for _ in 0..fasta_index.filled_line_count {
            let _ = self.fasta_buf_reader.read_exact(&mut one_line_buffer);
            self.sequence_buffer.extend_from_slice(&one_line_buffer);
            self.fasta_buf_reader.consume(self.line_terminator_size); // TODO: Apply const for better performance
        }

        // last line
        let _ = self.fasta_buf_reader.read_exact(&mut one_line_buffer);
        self.sequence_buffer.extend_from_slice(&one_line_buffer[..fasta_index.length_of_last_line]);
    }
}

impl Serialize for IndexedFastaProvider {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
            S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("IndexedFastaProvider", 5)?;
        state.serialize_field("total_record_count", &self.total_record_count)?;
        state.serialize_field("line_terminator_size", &self.line_terminator_size)?;
        state.serialize_field("use_reverse_complement", &self.use_reverse_complement)?;
        state.serialize_field("fasta_indices", &self.fasta_indices)?;
        state.serialize_field("fasta_file_path", &self.fasta_file_path)?;
        state.end()
    }
}

fn parse_label_from_line(line: String) -> String {
    let trimmed_description = line.strip_prefix(">").unwrap();
    let label = trimmed_description.trim().split(" ").next()
        .expect("No description in fasta file,")
        .to_string();
    label
}

#[derive(Debug, Serialize, Deserialize)]
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

struct FastaBufReader {
    buf_reader: BufReader<File>,
    buffer: String,
}
impl FastaBufReader {
    fn new<P: AsRef<Path> + std::fmt::Debug>(file_path: P) -> Result<Self> {
        let file = File::open(file_path)?;
        Ok(Self {
            buf_reader: BufReader::new(file),
            buffer: String::new(),
        })
    }
}

impl Iterator for FastaBufReader {
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

#[cfg(test)]
mod tests {
    use std::io::Read;

    use crate::*;
    use super::*;
    use crate::tests::sample_data::{
        NUCLEOTIDE_ONLY_FA_PATH_1,
        NUCLEOTIDE_ONLY_FA_PATH_2,
        FA_ENDS_WITH_CRLF_PATH,
        SIMPLE_FA_PATH,
    };

    #[test]
    fn print_indexed_fasta_provider() {
        let mut sequence_provider = IndexedFastaProvider::new(SIMPLE_FA_PATH).unwrap();

        {
            let label = sequence_provider.label_of_record(0);
            println!("{:?}", label);
        }

        {
            let seq = sequence_provider.sequence_of_record(0);
            println!("{:?}", String::from_utf8(seq.to_vec()));
        }
    }
}