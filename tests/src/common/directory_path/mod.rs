use cargo_metadata::MetadataCommand;
use std::path::PathBuf;
use anyhow::{anyhow, Result};

pub fn get_target_dir() -> Result<PathBuf> {
    let metadata = MetadataCommand::new()
        .exec()?;
    let target_dir = metadata.target_directory.as_std_path().to_path_buf();
    Ok(target_dir)
}
pub fn get_root_dir() -> Result<PathBuf> {
    let metadata = MetadataCommand::new()
        .exec()?;
    let mut test_package = None;
    for package in metadata.workspace_packages() {
        if package.name == "sigalign-tests" {
            test_package = Some(package.clone());
        }
    }
    let test_package = test_package.ok_or_else(|| anyhow!("Test package not found"))?;

    let path = test_package.manifest_path.as_std_path().parent()
        .ok_or_else(|| anyhow!("Root directory not found"))?
        .to_path_buf();
    Ok(path)
}
