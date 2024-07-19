use std::io::{Write, Read};

use base64::{Engine as _, engine::{general_purpose, GeneralPurpose}};
use thiserror::Error;
use capwriter::{Save, Load};

use sigalign_core::reference::{
    Reference as RawReference, extensions::Serialize,
};
use super::Reference;

const PREFIX: &str = "SIGALIGN_REFERENCE";
const LOWEST_COMPARABLE_WRAPPER_VERSION: &str = "0.4.0";
const CORE_VERSION: &str = "0.2.0";
const DELIMITER: &str = ":";

impl Reference {
    /// Save `Reference` to a writer.
    pub fn save_to<W>(&self, mut writer: W) -> Result<(), std::io::Error> where
        W: Write
    {
        let signature = Self::get_base64_encoded_signature_of_current_version();
        signature.as_bytes().save_to(&mut writer)?;
        self.raw_reference.save_to(writer)?;
        Ok(())
    }
    /// Load `Reference` from a reader.
    pub fn load_from<R>(mut reader: R) -> Result<Self, ReferenceLoadError> where
        R: Read,
        Self: Sized
    {
        let encoded_signature: Vec<u8> = Vec::load_from(&mut reader)?;
        let signatures = Self::get_base64_decoded_signature(&encoded_signature)?;
        if signatures[0] == PREFIX && signatures[1] == LOWEST_COMPARABLE_WRAPPER_VERSION && signatures[2] == CORE_VERSION {
            let raw_reference = RawReference::load_from(reader)?;
            Ok(Self::from(raw_reference))
        } else {
            Err(ReferenceLoadError::IncompatibleVersion(signatures[1].clone()))
        }
    }
    fn get_base64_encoded_signature_of_current_version() -> String {
        let engine = Self::get_base64_engine();
        let combined_signature = [PREFIX, DELIMITER, LOWEST_COMPARABLE_WRAPPER_VERSION, DELIMITER, CORE_VERSION].concat();
        let mut encoded_signature = String::new();
        engine.encode_string(combined_signature, &mut encoded_signature);

        encoded_signature
    }
    fn get_base64_decoded_signature(encoded_signature: &[u8]) -> Result<Vec<String>, ReferenceLoadError> {
        let engine = Self::get_base64_engine();
        if let Ok(decoded) = engine.decode(encoded_signature) {
            if let Ok(string) = String::from_utf8(decoded) {
                let components: Vec<String> = string.split(':').map(|x| x.to_string()).collect();
                return Ok(components)
            }
        }

        Err(ReferenceLoadError::UnknownFile)
    }
    fn get_base64_engine() -> GeneralPurpose {
        general_purpose::STANDARD_NO_PAD
    }
}

/// Error for loading `Reference`.
#[derive(Debug, Error)]
pub enum ReferenceLoadError {
    #[error("Unknown file format. The file does not appear to be a SigAlign reference file.")]
    UnknownFile,
    #[error("This reference file is incompatible with the current version of SigAlign. Detected version: {0}")]
    IncompatibleVersion(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
