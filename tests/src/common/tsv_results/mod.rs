use anyhow::Result;
use std::io::{BufRead, Read, Write};

use sigalign::results::{Alignment, AlignmentOperation, AlignmentOperations, AlignmentPosition, QueryAlignment, TargetAlignment};

pub fn write_query_alignment_as_tsv_format<W: Write>(
    query_index: u32,
    query_alignment: &QueryAlignment,
    mut writer: W,
) -> Result<()> {
    for target_alignment in &query_alignment.0 {
        let target_index = target_alignment.index;
        for alignment in &target_alignment.alignments {
            writer.write(format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                query_index,
                target_index,
                alignment.penalty,
                alignment.length,
                alignment.position.query.0,
                alignment.position.query.1,
                alignment.position.target.0,
                alignment.position.target.1,
                ops_to_string(&alignment.operations),
            ).as_bytes())?;
        }
    }
    Ok(())
}
pub fn read_query_alignments_from_tsv_format<R: Read>(
    total_query_num: u32,
    reader: R,
) -> Result<Vec<Option<QueryAlignment>>> {
    let mut buf_reader = std::io::BufReader::new(reader);
    let mut line = String::new();

    // key: query_index
    // value: AHashMap
    //   - key: target_index
    //   - value: Vec<Alignment>
    let mut alignments_map = ahash::AHashMap::new();

    while let Ok(_) = buf_reader.read_line(&mut line) {
        let mut fields = line.split('\t');
        let query_index = fields.next().unwrap().parse::<u32>()?;
        let target_index = fields.next().unwrap().parse::<u32>()?;
        let penalty = fields.next().unwrap().parse::<u32>()?;
        let length = fields.next().unwrap().parse::<u32>()?;
        let query_start = fields.next().unwrap().parse::<u32>()?;
        let query_end = fields.next().unwrap().parse::<u32>()?;
        let target_start = fields.next().unwrap().parse::<u32>()?;
        let target_end = fields.next().unwrap().parse::<u32>()?;
        let operations = string_to_ops(fields.next().unwrap());
        let alignment = Alignment {
            penalty,
            length,
            position: AlignmentPosition {
                query: (query_start, query_end),
                target: (target_start, target_end),
            },
            operations,
        };

        let target_alignment_map = alignments_map
            .entry(query_index)
            .or_insert_with(|| ahash::AHashMap::new());
        let alignments = target_alignment_map
            .entry(target_index)
            .or_insert_with(|| Vec::new());
        alignments.push(alignment);

        line.clear();
    }
    
    let mut query_alignments = Vec::new();
    for query_index in 0..total_query_num {
        match alignments_map.remove(&query_index) {
            Some(target_alignment_map) => {
                let mut target_alignments = Vec::new();
                for (target_index, alignments) in target_alignment_map {
                    target_alignments.push(TargetAlignment {
                        index: target_index,
                        alignments,
                    });
                }
                query_alignments.push(Some(QueryAlignment(target_alignments)));
            },
            None => query_alignments.push(None),
        }
    }
    Ok(query_alignments)
}   

fn ops_to_string(ops: &Vec<AlignmentOperations>) -> String {
    ops.iter().map(|op| format!("{}{}", op.count, match op.operation {
        AlignmentOperation::Match => "=",
        AlignmentOperation::Subst => "X",
        AlignmentOperation::Insertion => "I",
        AlignmentOperation::Deletion => "D",
    })).collect::<String>()
}
fn string_to_ops(s: &str) -> Vec<AlignmentOperations> {
    let mut ops = Vec::new();
    let mut count = 0;
    let mut op = AlignmentOperation::Match;
    for c in s.chars() {
        match c {
            '0'..='9' => {
                count = count * 10 + c.to_digit(10).unwrap();
            },
            '=' => {
                ops.push(AlignmentOperations { operation: op, count });
                count = 0;
                op = AlignmentOperation::Match;
            },
            'X' => {
                ops.push(AlignmentOperations { operation: op, count });
                count = 0;
                op = AlignmentOperation::Subst;
            },
            'I' => {
                ops.push(AlignmentOperations { operation: op, count });
                count = 0;
                op = AlignmentOperation::Insertion;
            },
            'D' => {
                ops.push(AlignmentOperations { operation: op, count });
                count = 0;
                op = AlignmentOperation::Deletion;
            },
            _ => panic!("Invalid character in alignment operations string: {}", c),
        }
    }
    ops
}
