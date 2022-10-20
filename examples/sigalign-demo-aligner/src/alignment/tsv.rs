use std::path::PathBuf;
use std::io::{Stdout, Write};

use super::{Result, format_err, error_msg};

use sigalign::util::{
    FastaReader,
    reverse_complement_of_nucleotide_sequence,
};
use sigalign::{
    Reference,
    core::ReferenceInterface,
    sequence_storage::SequenceStorage,
    Aligner,
};
use sigalign::{
    result::RecordAlignmentResult,
};

// TSV line format:
// | query label | reference index | record index | penalty | length |
//  query start position | query end position | record start position | record end position |
//  string operations |
pub fn alignment_as_tsv_to_stdout<S: SequenceStorage>(
    aligner: &mut Aligner,
    reference_index: usize,
    reference: Reference<S>,
    fasta_file_path: &PathBuf,
    stdout: &mut Stdout,
) {
    let mut sequence_buffer = reference.get_buffer();
    
    let fasta_reader = FastaReader::from_file_path(fasta_file_path).unwrap();
    fasta_reader.for_each(|(label, query)| {
        #[cfg(not(feature = "revcom"))]
        {
            let result = aligner.alignment(&reference, &mut sequence_buffer, &query);

            result.0.into_iter().for_each(|RecordAlignmentResult {
                index: record_index,
                alignments: anchor_results,
            }| {
                anchor_results.into_iter().for_each(|anchor_result| {
                    let line = format!(
                        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                        label, reference_index, record_index, anchor_result.penalty, anchor_result.length,
                        anchor_result.position.query.0, anchor_result.position.query.1,
                        anchor_result.position.record.0, anchor_result.position.record.1,
                        operations_to_string(&anchor_result.operations)
                    );
    
                    stdout.write(line.as_bytes()).unwrap();
                });
            });
        }
        #[cfg(feature = "revcom")]
        {
            let rev_com_query = reverse_complement_of_nucleotide_sequence(&query);

            // Org Query
            let result = aligner.alignment(&reference, &mut sequence_buffer, &query);
            result.0.into_iter().for_each(|RecordAlignmentResult {
                index: record_index,
                alignments: anchor_results,
            }| {
                anchor_results.into_iter().for_each(|anchor_result| {
                    let line = format!(
                        "{}\tF\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                        label, reference_index, record_index, anchor_result.penalty, anchor_result.length,
                        anchor_result.position.query.0, anchor_result.position.query.1,
                        anchor_result.position.record.0, anchor_result.position.record.1,
                        operations_to_string(&anchor_result.operations)
                    );
    
                    stdout.write(line.as_bytes()).unwrap();
                });
            });

            // Rc Query
            let result = aligner.alignment(&reference, &mut sequence_buffer, &rev_com_query);
            result.0.into_iter().for_each(|RecordAlignmentResult {
                index: record_index,
                alignments: anchor_results,
            }| {
                anchor_results.into_iter().for_each(|anchor_result| {
                    let line = format!(
                        "{}\tR\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                        label, reference_index, record_index, anchor_result.penalty, anchor_result.length,
                        anchor_result.position.query.0, anchor_result.position.query.1,
                        anchor_result.position.record.0, anchor_result.position.record.1,
                        operations_to_string(&anchor_result.operations)
                    );
    
                    stdout.write(line.as_bytes()).unwrap();
                });
            });
        }
    });
}

use sigalign::core::{
    AlignmentOperation,
    AlignmentCase,
};
fn operations_to_string(operations: &Vec<AlignmentOperation>) -> String {
    let string_ops: Vec<String> = operations.iter().map(|op| {
        format!(
            "{}{}",
            match op.case {
                AlignmentCase::Match => 'M',
                AlignmentCase::Subst => 'S',
                AlignmentCase::Insertion => 'I',
                AlignmentCase::Deletion => 'D',
            },
            op.count,
        )
    }).collect();
    string_ops.concat()
}