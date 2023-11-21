use std::{path::Path, borrow::Cow};
use std::marker::PhantomData;
use std::io::Read;
use std::fs::File;

use seq_io::fasta::{
    Reader as SeqIoReader,
    Record, RefRecord,
};

use super::{
    SeqRecord,
    IdRecord,
};

/// The reader of FASTA formatted file
pub struct FastaReader<R: Read> {
    reader: SeqIoReader<R>,
}

pub struct FastaRecord<'a> {
    record: RefRecord<'a>,
}

impl<'a, R: Read> FastaReader<R> {
    pub fn next(&'a mut self) -> Option<FastaRecord<'a>> {
        if let Some(Ok(seq)) = self.reader.next() {
            Some(FastaRecord {
                record: seq,
            })
        } else {
            None
        }
    }
}

// impl<R: Read> FastaReader<R> {
//     pub fn new(reader: R) -> Self {
//         let reader = SeqIoReader::new(reader);
//         while let Some(record) = reader.next() {
//             let record = record.expect("Error reading record");
//             for s in record.seq_lines() {
//                 sum += s.len();
//             }
//             n += 1;
//         }
//         let seq = reader.next();
//         Self {
//             reader
//         }
//     }
// }

// impl<R: Read> Iterator for FastaReader<R> {
//     type Item = (String, Cow<'a, [u8]>);

//     fn next(&mut self) -> Option<Self::Item> {
//         match self.reader.next() {
//             Some(Ok(seq)) => {
//                 let seq = seq.full_seq();

//                 Some((
//                     String::from_utf8(seq.id_bytes().to_vec()).unwrap(),
//                     seq.to_owned_record().seq,
//                 ))
//             },
//             _ => {
//                 None
//             },
//         }
//     }
// }
