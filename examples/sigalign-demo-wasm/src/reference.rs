use crate::{Result, error_msg};
use super::{SigReference, SigReferenceBuilder, InMemoryStorage};
use super::{Deserialize, Serialize, serialize_to_string};

// Reference
pub struct ReferenceDep {
    pub sig_reference: Option<SigReference<InMemoryStorage>>,
    state: ReferenceState,
}

#[derive(Serialize)]
struct ReferenceState {
    exist: bool,
    // Sequence type
    allowed_characters: String,
    is_nucleotide: bool,
    noise_character: String,
    // Pattern finder
    sampling_ratio: usize,
    bwt_block_size: usize,
    lookup_table_kmer_size: usize,
}
impl Default for ReferenceDep {
    fn default() -> Self {
        Self {
            sig_reference: None,
            state: ReferenceState::default(),
        }
    }
}
impl Default for ReferenceState {
    fn default() -> Self {
        Self {
            exist: false,
            allowed_characters: "".to_string(),
            is_nucleotide: true,
            noise_character: "".to_string(),
            sampling_ratio: 0,
            bwt_block_size: 0,
            lookup_table_kmer_size: 0,
        }
    }
}
impl ReferenceDep {
    pub fn generate_with_fasta_bytes(
        &mut self,
        fasta_bytes: &str,
        sampling_ratio: usize,
        bwt_block: usize,
        lookup_table_kmer_size: usize,
    ) -> Result<()> {
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta_bytes(fasta_bytes.as_bytes());
        self.generate_with_sequence_storage(sequence_storage, sampling_ratio, bwt_block, lookup_table_kmer_size)?;

        Ok(())
    }
    pub fn generate_with_fasta_file(
        &mut self,
        fasta_file_path: &str,
        sampling_ratio: usize,
        bwt_block: usize,
        lookup_table_kmer_size: usize,
    ) -> Result<()> {
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta_file(fasta_file_path)?;
        self.generate_with_sequence_storage(sequence_storage, sampling_ratio, bwt_block, lookup_table_kmer_size)?;

        Ok(())
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn state_to_json(&self) -> String {
        match serialize_to_string(&self.state) {
            Ok(json) => json,
            Err(error) => format!("{{\"error\": {}}}", error),
        }
    }

    fn generate_with_sequence_storage(
        &mut self,
        sequence_storage: InMemoryStorage,
        sampling_ratio: usize,
        bwt_block: usize,
        lookup_table_kmer_size: usize,
    ) -> Result<()> {
        let mut sig_reference_builder = SigReferenceBuilder::new()
        .change_sampling_ratio(sampling_ratio as u64)?
            .change_count_array_kmer(lookup_table_kmer_size)?;

        if bwt_block == 64 {
            sig_reference_builder = sig_reference_builder.change_bwt_block_size_to_64();
        } else if bwt_block == 128 {
            sig_reference_builder = sig_reference_builder.change_bwt_block_size_to_128();
        } else {
            error_msg!("Bwt block only allow 64 or 128")
        }

        let sig_reference = sig_reference_builder.build(sequence_storage)?;

        let state = {
            // pattern finder
            let sampling_ratio = sig_reference.get_suffix_array_sampling_ratio() as usize;
            let bwt_block_size = sig_reference.get_size_of_bwt_block();
            let lookup_table_kmer_size = sig_reference.get_lookup_table_kmer_size();
            
            // sequence type
            let text_type_is_nucleotide = sig_reference.get_whether_text_type_is_nucleotide();
            let noise_character = match sig_reference.get_noise_character_of_text_type() {
                Some(chr) => String::from_utf8(vec![chr])?,
                None => "".to_string(),
            };
            let allowed_characters = String::from_utf8(sig_reference.get_allowed_character_list().to_vec())?;

            ReferenceState {
                exist: true,
                allowed_characters,
                is_nucleotide: text_type_is_nucleotide,
                noise_character,
                sampling_ratio,
                bwt_block_size,
                lookup_table_kmer_size,
            }
        };

        self.sig_reference = Some(sig_reference);
        self.state = state;

        Ok(())
    }
}