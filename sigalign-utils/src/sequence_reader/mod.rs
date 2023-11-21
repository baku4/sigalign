pub mod fasta;
pub mod fastq;

pub trait SeqReader<'a> {
    fn fill_next(&mut self, buf: &'a mut Vec<u8>) -> Option<&'a [u8]>;
    fn extend_next(&mut self, buf: &'a mut Vec<u8>) -> Option<&'a [u8]>;
}

pub trait SeqRecord {
    fn fill_buf(&mut self, buf: &mut Vec<u8>);
    fn extend_buf(&mut self, buf: &mut Vec<u8>);
}
pub trait IdRecord {
    fn fill_buf(&mut self, buf: &mut Vec<u8>);
    fn extend_buf(&mut self, buf: &mut Vec<u8>);
}

pub struct BufferedSequence {
    seq_buffer: Vec<u8>,
}
impl<'a> BufferedSequence {
    pub fn new() -> Self {
        Self { seq_buffer: Vec::new() }
    }
    pub fn seq(&self) -> &[u8] {
        &self.seq_buffer
    }
    pub fn parse_next<R: SeqRecord>(&'a mut self, record: &mut R) -> Option<&'a [u8]> {
        record.fill_buf(&mut self.seq_buffer);
        Some(self.seq())
    }
}

pub struct BufferedSequenceWithId {
    seq_buffer: Vec<u8>,
    id_buffer: Vec<u8>,
}
impl BufferedSequenceWithId {
    pub fn new() -> Self {
        Self { seq_buffer: Vec::new(), id_buffer: Vec::new() }
    }
    pub fn seq(&self) -> &[u8] {
        &self.seq_buffer
    }
    pub fn id(&self) -> &[u8] {
        &self.id_buffer
    }
}

// fn test() {
//     let mut buf_seq = BufferedSequence::new();
//     let mut fasta_reader = FastaReader::new();

//     // (1)
//     while let Some(seq) = fasta_reader.fill_next(&mut buf_seq) {
//         //
//     }

//     // (2)
//     while let Some(record) = fasta_reader.next() {
//         record.fill_buf(&mut buf_seq);
//         //
//     }
// }
