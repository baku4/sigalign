use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference, SequenceProvider, JoinedSequence,
    SequenceType, PatternFinder,
    // traits
    Divisible, Serializable, SizeAware,
    LabelProvider,
    ReverseComplement,
};

use super::{InMemoryProvider, InMemoryBuffer};

use crate::util::{FastaReader, reverse_complement_of_nucleotide_sequence};

use serde::{Serialize, Deserialize};
use bincode::{serialize_into, deserialize_from};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InMemoryRcProvider(InMemoryProvider);

impl InMemoryRcProvider {
    pub fn new() -> Self {
        Self(InMemoryProvider::new())
    }
    pub fn add_record(
        &mut self,
        sequence: &[u8],
        label: &str,
    ) {
        self.0.add_record(sequence, label);
        let rc_seq = reverse_complement_of_nucleotide_sequence(sequence);
        self.0.add_record(&rc_seq, label);
    }
    pub fn add_fasta_file<P>(&mut self, file_path: P) -> Result<()> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_file_path(file_path)?;
        self.add_from_fasta_reader(fasta_reader);
        Ok(())
    }
    pub fn add_fasta_bytes(&mut self, fasta_bytes: &[u8]) {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        self.add_from_fasta_reader(fasta_reader);
    }
    fn add_from_fasta_reader<R>(&mut self, fasta_reader: FastaReader<R>) where
        R: std::io::Read,
    {
        fasta_reader.for_each(|(label, sequence)| {
            self.0.add_record(&sequence, &label);
            let rc_seq = reverse_complement_of_nucleotide_sequence(&sequence);
            self.0.add_record(&rc_seq, &label);
        });
    }
}

// Sequence Provider
impl SequenceProvider for InMemoryRcProvider {
    type Buffer = InMemoryBuffer;

    fn total_record_count(&self) -> usize {
        self.0.record_count
    }
    fn get_buffer(&self) -> Self::Buffer {
        self.0.get_buffer()
    }
    fn fill_sequence_buffer(&self, record_index: usize, buffer: &mut Self::Buffer) {
        self.0.fill_sequence_buffer(record_index, buffer);
    }
    fn get_joined_sequence(&self) -> JoinedSequence {
        self.0.get_joined_sequence()
    }
}

// Label Provider
impl LabelProvider for InMemoryRcProvider {
    fn label_of_record(&self, record_index: usize) -> String {
        self.0.label_of_record(record_index)
    }
}

// Serializable
impl Serializable for InMemoryRcProvider {
    fn save_to<W>(&self, writer: W) -> Result<()> where
        W: std::io::Write
    {
        self.0.save_to(writer)?;
        Ok(())
    }
    fn load_from<R>(reader: R) -> Result<Self> where
        R: std::io::Read,
        Self: Sized,
    {
        let in_memory_provider = InMemoryProvider::load_from(reader)?;
        Ok(Self(in_memory_provider))
    }
}

// SizeAware
impl SizeAware for InMemoryRcProvider {
    fn size_of(&self) -> usize {
        self.0.size_of()
    }
}

// Divisible
impl Divisible for InMemoryRcProvider {
    fn split_by_max_length(self, max_seq_len: usize) -> Result<Vec<Self>> {
        // Get record index range list
        let record_index_range_list = self.0.record_index_range_list_of_max_length(max_seq_len);

        // Adjust all range to even number
        let adjusted_record_index_range_list = record_index_range_list.into_iter()
            .filter_map(|(mut v1, mut v2)| {
                if v1 % 2 == 1 {
                    v1 += 1;
                }
                if v2 % 2 == 1 {
                    v2 += 1;
                }
                if v1 == v2 {
                    None
                } else {
                    Some((v1, v2))
                }
            }).collect();
        
        // Split
        let splitted = self.0.split_using_record_index_range_list(adjusted_record_index_range_list).into_iter()
            .map(|inner| {
                Self(inner)
            }).collect();

        Ok(splitted)
    }
}

// Reverse Complement
impl ReverseComplement for InMemoryRcProvider {
    fn is_reverse_complement(&self, record_index: usize) -> bool {
        if record_index % 2 == 0 {
            false
        } else {
            true
        }
    }
}
