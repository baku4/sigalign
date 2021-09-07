use std::default;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use super::SequenceProvider;
use crate::{Result, anyhow};

use bio::io::fasta::{Index, IndexedReader};

pub struct FastaProvider<'a> {
    fasta_path: &'a str,
    index_reader: IndexedReader<File>,
    record_count: usize,
}
impl<'a> FastaProvider<'a> {
    fn new(fasta_path: &'a str) -> Result<Self> {
        let (fai, record_count) = generate_fai_and_count(fasta_path)?;
        let fasta = File::open(fasta_path)?;
        let index_reader = IndexedReader::new(fasta, &*fai)?;

        Ok(Self {
            fasta_path,
            index_reader,
            record_count,
        })
    }
}


#[derive(Debug, Default, Clone)]
struct FaiOneline {
    name: String,
    length: usize,
    offset: usize,
    linebases: usize,
    linewidth: usize,
}
impl FaiOneline {
    fn new() -> Self {
        Self::default()
    }
}

struct FastaBufReader {
    buf_reader: BufReader<File>,
    buffer: String,
}
impl FastaBufReader {
    fn new(fasta_file: &str) -> Result<Self> {
        let f = File::open(fasta_file)?;
        Ok(Self {
            buf_reader: io::BufReader::new(f),
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

// https://doc.rust-lang.org/rust-by-example/std_misc/file/read_lines.html
fn read_fasta(fasta_path: &str) -> Result<()> {
    let f = File::open(fasta_path)?;
    let mut buf_reader = io::BufReader::new(f);
    let mut buf = String::new();

    Ok(())
}

fn generate_fai_and_count(fasta_path: &str) -> Result<(Vec<u8>, usize)> {
    let mut fai: Vec<FaiOneline> = Vec::new();
    let mut count = 0;

    let mut fasta_buf_reader = FastaBufReader::new(fasta_path)?;

    let mut fai_oneline: FaiOneline = FaiOneline::new();
    let mut pre_offset = 0;
    let mut pre_is_name = false;
    let mut length = 0;
    // Init with first line
    let lt = if let Some(l) = fasta_buf_reader.next() {
        count += 1;
        let name = get_name_from_desc(l.clone())?;
        fai_oneline.name = name;

        fai_oneline.offset += l.len();
        if l.ends_with("\r\n") {
            LineTerminator::CRLF
        } else if l.ends_with("\n") {
            LineTerminator::LF
        } else {
            return Err(anyhow!("Not a valid fasta file"));
        }
    } else {
        return Err(anyhow!("Not a valid fasta file"));
    };
    pre_is_name = true;
    pre_offset = fai_oneline.offset;
    
    while let Some(l) = fasta_buf_reader.next() {
        count += 1;
        if l.starts_with(">") { // line is desc
            // push last fai
            fai_oneline.length = length;
            fai.push(fai_oneline.clone());
            // deal with this line
            let desc_len = l.len();
            let name = get_name_from_desc(l)?;
            fai_oneline.name = name;
            pre_offset += desc_len;
            fai_oneline.offset = pre_offset;
            pre_is_name = true;
            length = 0;
        } else { // line is of sequence
            let seq_len = l.len();
            if pre_is_name {
                fai_oneline.linebases = seq_len - lt.last_bytes();
                fai_oneline.linewidth = seq_len;
                pre_is_name = false;
            }
            pre_offset += seq_len;
            length += seq_len - lt.last_bytes();
        }
    }
    // push last fai
    fai_oneline.length = length;
    fai.push(fai_oneline.clone());

    // fai to bytes
    let fai_bytes = fai.into_iter().map(|fai_oneline| {
        format!("{}\t{}\t{}\t{}\t{}", fai_oneline.name, fai_oneline.length, fai_oneline.offset, fai_oneline.linebases, fai_oneline.linewidth)
    }).collect::<Vec<String>>().join("\n").as_bytes().to_vec();
    Ok((fai_bytes, count))
}

fn get_name_from_desc(l: String) -> Result<String> {
    let trimmed_name = if let Some(v) = l.strip_prefix(">") {
        v.trim()
    } else {
        return Err(anyhow!("Not a valid fasta file 4"));
    };
    Ok(trimmed_name.split(" ").next().unwrap().to_string())
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

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;
    
    #[test]
    fn test_generate_fai() {
        let fai_bytes = generate_fai_and_count("./src/tests/fasta/ERR209055.fa").unwrap();
        println!("{:?}", String::from_utf8(fai_bytes.0).unwrap());
    }
}