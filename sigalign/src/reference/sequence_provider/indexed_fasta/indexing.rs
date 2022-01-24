use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceProvider, JoinedSequence,
    SequenceType, PatternFinder,
    Serializable,
    LabelProvider,
};
use super::{FastaIndex, LineBufReader};

const LF_TERMINATION_SIZE: usize = 1;
const CRLF_TERMINATION_SIZE: usize = 2;

impl FastaIndex {
    pub fn get_indices_and_line_terminator_size(line_buf_reader: &mut LineBufReader) -> Result<(Vec::<Self>, usize)> {
        let mut fasta_indices = Vec::new();

        let mut offset_to_current_line = 0;
        let mut offset_to_sequence_start_point = 0;
        let mut sequence_length = 0;
        let mut sequence_length_of_lines = Vec::new();
        let mut label = String::new();

        // Init with first line (This line is always description)
        let line_terminator_size = if let Some(line) = line_buf_reader.next() {
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

            label = Self::parse_label_from_line(line);

            line_terminator
        } else {
            error_msg!("No record in fasta file.");
        };

        while let Some(line) = line_buf_reader.next() {
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
                
                label = Self::parse_label_from_line(line);
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

        Ok((fasta_indices, line_terminator_size))
    }
    fn parse_label_from_line(line: String) -> String {
        let trimmed_description = line.strip_prefix(">").unwrap();
        let label = trimmed_description.trim().split(" ").next()
            .expect("No description in fasta file,")
            .to_string();
        label
    }
}

use std::io::BufReader;
use std::fs::File;
