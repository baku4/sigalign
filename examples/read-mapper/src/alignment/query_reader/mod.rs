use std::{fs::File, io::Read, path::PathBuf};

use crate::{error, Result};

use sigalign_utils::sequence_reader::{
    decompress::get_gzip_decoder, fasta::FastaReader, fastq::FastqReader, IdRecord, SeqRecord as _,
};

pub enum QueryReader {
    Fasta(FastaReader<Box<dyn Read + Send>>),
    Fastq(FastqReader<Box<dyn Read + Send>>),
}

impl QueryReader {
    pub fn new(file_path: &PathBuf, is_gzip_compressed: bool, is_fasta_file: bool) -> Result<Self> {
        let read: Box<dyn Read + Send> = {
            let file = File::open(file_path)?;
            if is_gzip_compressed {
                Box::new(get_gzip_decoder(file))
            } else {
                Box::new(file)
            }
        };

        if is_fasta_file {
            Ok(Self::Fasta(FastaReader::new(read)))
        } else {
            Ok(Self::Fastq(FastqReader::new(read)))
        }
    }
    pub fn fill_record_buffer(
        &mut self,
        sequence_buffer: &mut Vec<u8>,
        label_buffer: &mut String,
    ) -> Result<()> {
        match self {
            Self::Fasta(reader) => {
                if let Some(mut record) = reader.next() {
                    sequence_buffer.clear();
                    label_buffer.clear();
                    record.extend_seq_buf(sequence_buffer);
                    record.extend_id_string(label_buffer)?;
                    Ok(())
                } else {
                    Err(error!("No more records"))
                }
            }
            Self::Fastq(reader) => {
                if let Some(mut record) = reader.next() {
                    sequence_buffer.clear();
                    label_buffer.clear();
                    record.extend_seq_buf(sequence_buffer);
                    record.extend_id_string(label_buffer)?;
                    Ok(())
                } else {
                    Err(error!("No more records"))
                }
            }
        }
    }
}
