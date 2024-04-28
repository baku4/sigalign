use std::path::PathBuf;

use glob::glob;

use crate::{Result, error_msg, error};

#[derive(Debug, Clone)]
pub struct ReferencePathDetector {
    original_path: PathBuf,
}

impl ReferencePathDetector {
    pub fn new(original_path: &PathBuf) -> Self {
        Self {
            original_path: original_path.clone(),
        }
    }
    pub fn get_path_of_index(&self, index: usize) -> PathBuf {
        if index == 0 {
            self.original_path.clone()
        } else {
            let mut file_name = self.original_path.file_name().unwrap().to_os_string();
            file_name.push(format!(".c{}", index)); // c for chunk
            self.original_path.with_file_name(file_name)
        }
    }
    pub fn get_manifest_file_path(&self) -> PathBuf {
        let mut manifest_file_name = self.original_path.file_name().unwrap().to_os_string();
        manifest_file_name.push(".manifest");
        self.original_path.with_file_name(manifest_file_name)
    }
    pub fn to_clean_up_paths(&self) -> Result<Vec<PathBuf>> {
        let mut reference_paths = Vec::new();
        // Original reference file
        if self.original_path.exists() {
            reference_paths.push(self.original_path.clone());
        }

        // For chunked reference files
        let pattern = format!(
            "{}.c*",
            self.original_path.as_os_str().to_str().ok_or(
                error!("Failed to convert path to string: {:?}", self.original_path)
            )?,
        );

        let entries = glob(&pattern)?;
        for entry in entries {
            match entry {
                Ok(path) => {
                    reference_paths.push(path);
                }
                Err(e) => {
                    error_msg!("Failed to read glob pattern: {}", e);
                }
            }
        }

        // Manifest file
        let manifest_file_path = self.get_manifest_file_path();
        if manifest_file_path.exists() {
            reference_paths.push(manifest_file_path);
        }

        Ok(reference_paths)
    }
    pub fn load_reference_chunk_paths(&self) -> Result<Vec<PathBuf>> {
        let mut reference_chunk_paths = vec![self.original_path.clone()];

        let mut suffix_number = 1;
        loop {
            let mut suffix_added_file_name = self.original_path.file_name().ok_or(
                error!("Failed to get file name from path: {:?}", self.original_path)
            )?.to_os_string();
            suffix_added_file_name.push(format!(".c{}", suffix_number));

            let additional_ref_path = self.original_path.clone().with_file_name(suffix_added_file_name);
            if additional_ref_path.exists() {
                reference_chunk_paths.push(additional_ref_path);
                suffix_number += 1;
            } else {
                break;
            }
        }

        Ok(reference_chunk_paths)
    }
}
