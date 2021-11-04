// Generation of Fai format in Samtools
use crate::{Result, error_msg};

use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn fai_bytes_and_count_of_fasta_file<P>(file_path: P) -> Result<(Vec<u8>, usize)> where
    P: AsRef<Path> + std::fmt::Debug {
    let mut fai: Vec<FaiLine> = Vec::new();
    let mut count = 0;

    let mut fasta_buf_reader = FastaBufReader::new(file_path)?;

    let mut fai_one_line: FaiLine = FaiLine::new();
    let mut pre_offset = 0;
    let mut pre_is_name = false;
    let mut length = 0;
    // Init with first line
    let lt = if let Some(l) = fasta_buf_reader.next() {
        count += 1;
        let name = name_of_description(l.clone())?;
        fai_one_line.name = name;

        fai_one_line.offset += l.len();
        if l.ends_with("\r\n") {
            LineTerminator::CRLF
        } else if l.ends_with("\n") {
            LineTerminator::LF
        } else {
            error_msg!(""); // TODO: Write err msg
        }
    } else {
        error_msg!(""); // TODO: Write err msg
    };
    pre_is_name = true;
    pre_offset = fai_one_line.offset;
    
    while let Some(l) = fasta_buf_reader.next() {
        if l.starts_with(">") { // line is desc
            count += 1;
            // push last fai
            fai_one_line.length = length;
            fai.push(fai_one_line.clone());
            // deal with this line
            let desc_len = l.len();
            let name = name_of_description(l)?;
            fai_one_line.name = name;
            pre_offset += desc_len;
            fai_one_line.offset = pre_offset;
            pre_is_name = true;
            length = 0;
        } else { // line is of sequence
            let seq_len = l.len();
            if pre_is_name {
                fai_one_line.line_bases = seq_len - lt.last_bytes();
                fai_one_line.line_width = seq_len;
                pre_is_name = false;
            }
            pre_offset += seq_len;
            length += seq_len - lt.last_bytes();
        }
    }
    // push last fai
    fai_one_line.length = length;
    fai.push(fai_one_line.clone());

    // fai to bytes
    let fai_bytes = fai.into_iter().map(|fai_line| {
        format!("{}\t{}\t{}\t{}\t{}", fai_line.name, fai_line.length, fai_line.offset, fai_line.line_bases, fai_line.line_width)
    }).collect::<Vec<String>>().join("\n").as_bytes().to_vec();
    Ok((fai_bytes, count))
}

fn name_of_description(l: String) -> Result<String> {
    let trimmed_name = if let Some(v) = l.strip_prefix(">") {
        v.trim()
    } else {
        error_msg!(""); // TODO: Write err msg
    };
    Ok(trimmed_name.split(" ").next().unwrap().to_string())
}


#[derive(Debug, Default, Clone)]
struct FaiLine {
    name: String,
    length: usize,
    offset: usize,
    line_bases: usize,
    line_width: usize,
}
impl FaiLine {
    fn new() -> Self {
        Self::default()
    }
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
        self.buf_reader.read_line(&mut self.buffer);
        if self.buffer.len() == 0 {
            None
        } else {
            Some(self.buffer.clone())
        }
    }
}

enum LineTerminator {
    LF,
    CRLF,
}
impl LineTerminator {
    fn last_bytes(&self) -> usize {
        match self {
            Self::LF => 1,
            Self::CRLF => 2,
        }
    }
}
