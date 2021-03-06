use crate::{Result, error_msg};
use crate::{Serialize, Deserialize};
use super::{Reference, ReferenceProtoDep, SequenceProvider, Labeling, Writable, reverse_complement_of_nucleotide_sequence};

use std::path::Path;
use std::fs::File;
use std::io::{Read, BufRead, BufReader, Seek, SeekFrom, Write};

const LF_TERMINATION_SIZE: usize = 1;
const CRLF_TERMINATION_SIZE: usize = 2;

/// Basic implementation for [SequenceProvider] parsing sequences from fasta file using index.
/// This index allows random access to record label.
#[derive(Debug)]
pub struct IndexedFastaProvider {
    proto: IndexedFastaProviderProto,
    fasta_buf_reader: BufReader<File>,
    // Buffers
    sequence_buffer: Vec<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct IndexedFastaProviderProto {
    total_record_count: usize,
    line_terminator_size: usize,
    use_reverse_complement: bool,
    fasta_indices: Vec<FastaIndex>,
}
impl Writable for IndexedFastaProviderProto {}

impl SequenceProvider for IndexedFastaProvider {
    fn total_record_count(&self) -> usize {
        if self.proto.use_reverse_complement {
            self.proto.total_record_count * 2
        } else {
            self.proto.total_record_count
        }
    }
    fn sequence_of_record(&mut self, record_index: usize) -> &[u8] {
        if self.proto.use_reverse_complement {
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
        &self.proto.fasta_indices[record_index].label
    }
}
impl IndexedFastaProvider {
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(
        fasta_file_path: P,
    ) -> Result<Self> {
        let mut fasta_indices = Vec::new();

        let mut fasta_buf_reader = FastaBufReader::new(fasta_file_path)?;
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

        // Push last FastaIndex TODO: Deduplicate code
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

        Ok(
            Self {
                proto: IndexedFastaProviderProto {
                    total_record_count: fasta_indices.len(),
                    line_terminator_size: line_terminator_size,
                    use_reverse_complement: false,
                    fasta_indices: fasta_indices,
                },
                fasta_buf_reader: fasta_buf_reader.buf_reader,
                sequence_buffer: Vec::new(),
            }
        )        
    }
    fn fill_buffer_sequence_from_fasta(
        &mut self,
        record_index: usize,
    ) {
        let fasta_index = &self.proto.fasta_indices[record_index];

        self.sequence_buffer = Vec::with_capacity(fasta_index.sequence_length);

        let mut one_line_buffer: Vec<u8> = vec![0; fasta_index.length_of_one_line];

        self.fasta_buf_reader.seek(SeekFrom::Start(fasta_index.sequence_offset)).unwrap();

        // filled line
        for _ in 0..fasta_index.filled_line_count {
            let _ = self.fasta_buf_reader.read_exact(&mut one_line_buffer);
            self.sequence_buffer.extend_from_slice(&one_line_buffer);
            self.fasta_buf_reader.consume(self.proto.line_terminator_size); // TODO: Apply const for better performance
        }

        // last line
        let _ = self.fasta_buf_reader.read_exact(&mut one_line_buffer);
        self.sequence_buffer.extend_from_slice(&one_line_buffer[..fasta_index.length_of_last_line]);
    }
}


#[derive(Serialize, Deserialize)]
struct ReferenceWithIndexedFastaProviderFile {
    serialized_reference_proto: Vec<u8>,
    serialized_indexed_fasta_provider_proto: Vec<u8>
}

impl Writable for ReferenceWithIndexedFastaProviderFile {}

impl Reference<IndexedFastaProvider> {
    pub fn write_to<W: Write>(&self, writer: W) -> Result<()> {
        let file = ReferenceWithIndexedFastaProviderFile {
            serialized_reference_proto: bincode::serialize(
                &self.reference_proto
            )?,
            serialized_indexed_fasta_provider_proto: bincode::serialize(
                &self.sequence_provider.proto
            )?,
        };

        file.write_to(writer)
    }
    pub fn read_from<R: Read>(reader: R, fasta_file_path: &str) -> Result<Self> {
        let (reference_proto, indexed_fasta_provider_proto): (ReferenceProtoDep, IndexedFastaProviderProto) = {
            let file = ReferenceWithIndexedFastaProviderFile::read_from(reader)?;
            let reference_proto = bincode::deserialize(&file.serialized_reference_proto)?;
            let indexed_fasta_provider_proto = bincode::deserialize(&file.serialized_indexed_fasta_provider_proto)?;

            (reference_proto, indexed_fasta_provider_proto)
        };

        let fasta_file = File::open(fasta_file_path)?;
        let fasta_buf_reader = BufReader::new(fasta_file);

        let indexed_fasta_provider = IndexedFastaProvider {
            proto: indexed_fasta_provider_proto,
            fasta_buf_reader,
            sequence_buffer: Vec::new(),
        };
        let reference = Reference {
            reference_proto,
            sequence_provider: indexed_fasta_provider,
        };

        Ok(reference)
    }
}


fn parse_label_from_line(line: String) -> String {
    let trimmed_description = line.strip_prefix(">").unwrap();
    let label = trimmed_description.trim().split(" ").next()
        .expect("No description in fasta file,")
        .to_string();
    label
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        let mut sequence_provider = IndexedFastaProvider::new(NUCLEOTIDE_ONLY_FA_PATH_1).unwrap();

        {
            let label = sequence_provider.label_of_record(10);
            println!("{:?}", label);
        }

        {
            let seq = sequence_provider.sequence_of_record(10);
            println!("{:?}", String::from_utf8(seq.to_vec()));
        }
    }
}