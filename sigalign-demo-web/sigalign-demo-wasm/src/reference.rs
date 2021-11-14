use crate::{Result, error_msg};
use super::{SigReference, LtFmIndexConfig, InMemoryProvider};
use super::{Deserialize, Serialize, serialize_to_string};

// Reference
pub struct Reference {
    pub sig_reference: Option<SigReference<InMemoryProvider>>,
    state: ReferenceState,
}

#[derive(Serialize)]
struct ReferenceState {
    exist: bool,
    is_nucleotide: bool,
    noise_character: String,
    sampling_ratio: usize,
    bwt_block_size: usize,
    lookup_table_kmer_size: usize,
}
impl Default for Reference {
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
            is_nucleotide: true,
            noise_character: "".to_string(),
            sampling_ratio: 0,
            bwt_block_size: 0,
            lookup_table_kmer_size: 0,
        }
    }
}
impl Reference {
    pub fn generate_with_fasta_bytes(
        &mut self,
        fasta_bytes: &str,
        sampling_ratio: usize,
        bwt_block: usize,
        lookup_table_kmer_size: usize,
    ) -> Result<()> {
        let sequence_provider = InMemoryProvider::from_fasta_bytes(fasta_bytes.as_bytes());
        self.generate_with_sequence_provider(sequence_provider, sampling_ratio, bwt_block, lookup_table_kmer_size)?;

        Ok(())
    }
    pub fn generate_with_fasta_file(
        &mut self,
        fasta_file_path: &str,
        sampling_ratio: usize,
        bwt_block: usize,
        lookup_table_kmer_size: usize,
    ) -> Result<()> {
        let sequence_provider = InMemoryProvider::from_fasta_file(fasta_file_path)?;
        self.generate_with_sequence_provider(sequence_provider, sampling_ratio, bwt_block, lookup_table_kmer_size)?;

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

    fn generate_with_sequence_provider(
        &mut self,
        sequence_provider: InMemoryProvider,
        sampling_ratio: usize,
        bwt_block: usize,
        lookup_table_kmer_size: usize,
    ) -> Result<()> {
        let mut lt_fm_index_config = LtFmIndexConfig::new()
            .change_sampling_ratio(sampling_ratio as u64)
            .change_kmer_size_for_lookup_table(lookup_table_kmer_size);
        if bwt_block == 64 {
            // use default
        } else if bwt_block == 128 {
            lt_fm_index_config = lt_fm_index_config.use_bwt_size_of_128();
        } else {
            error_msg!("Bwt block only allow 64 or 128")
        }

        let sig_reference = SigReference::new_with_lt_fm_index_config(lt_fm_index_config, sequence_provider)?;

        let state = {
            let sampling_ratio = sig_reference.get_sa_sampling_ratio() as usize;
            let bwt_block_size = sig_reference.get_size_of_bwt_block();
            let lookup_table_kmer_size = sig_reference.get_kmer_size_for_lookup_table();
            let sequence_type = sig_reference.get_sequence_type();

            let noise_character = match sequence_type.1 {
                Some(character) => {
                    let string = vec![character];
                    String::from_utf8(string).unwrap()
                },
                None => {
                    "".to_string()
                },
            };

            ReferenceState {
                exist: true,
                is_nucleotide: sequence_type.0,
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