use std::path::PathBuf;
use anyhow::{Result, bail};

use crate::common::{
    directory_path::get_target_dir,
    test_data::DataForValidation,
};

mod generate_results_with_multiple_threads;

#[derive(Debug, Clone)]
pub struct DpmTestUnit {
    // Test File
    test_data: DataForValidation,
    // Test Spec
    mode: DpmTestMode,
    px: u32,
    po: u32,
    pe: u32,
    minl: u32,
    maxp: f32,
    // Results cache
    result_cache_file: PathBuf,
}

impl DpmTestUnit {
    // New
    pub fn new(
        mode: DpmTestMode,
        test_data: DataForValidation,
        px: u32, po: u32, pe: u32,
        minl: u32, maxp: f32,
    ) -> Result<Self> {
        let data_tag = test_data.get_tag();
        let mode_tag = mode.get_tag();
        let result_cache_file = Self::get_result_cache_file(
            data_tag, mode_tag, px, po, pe, minl, maxp,
        )?;
        Ok(Self {
            test_data,
            mode,
            px, po, pe,
            minl, maxp,
            result_cache_file,
        })
    }
    pub fn from_cached_files() -> Result<Vec<Self>> {
        let cache_file_dir = Self::get_cache_file_dir()?;
        let cached_test_units = cache_file_dir.read_dir()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                let fname = path.file_name()?.to_str()?;
                let parts: Vec<&str> = fname.split('-').collect();
                let data_tag = parts[0];
                let mode_tag = parts[1];
                let px = parts[2].parse().ok()?;
                let po = parts[3].parse().ok()?;
                let pe = parts[4].parse().ok()?;
                let minl = parts[5].parse().ok()?;
                let maxp = parts[6].parse().ok()?;
                let test_data = DataForValidation::from_tag(data_tag).ok()?;
                let mode = DpmTestMode::from_tag(mode_tag).ok()?;
                Some(Self {
                    test_data,
                    mode,
                    px, po, pe,
                    minl, maxp,
                    result_cache_file: path,
                })
            })
            .collect();
        Ok(cached_test_units)
    }

    // Helpers
    fn get_cache_file_dir() -> Result<PathBuf> {
        let target_dir = get_target_dir()?;
        let path = target_dir.join("dpm_validation_cache");
        if !path.exists() {
            std::fs::create_dir(&path)?;
        }
        Ok(path)
    }
    fn get_result_cache_file(
        data_tag: &str,
        mode_tag: &str,
        px: u32,
        po: u32,
        pe: u32,
        minl: u32,
        maxp: f32,
    ) -> Result<PathBuf> {
        let path = Self::get_cache_file_dir()?;
        let scaled_maxp = (maxp * 10_000.0) as u32;
        let fname = format!(
            "{}-{}-{}-{}-{}-{}-{}.tsv",
            data_tag,
            mode_tag,
            px,
            po,
            pe,
            minl,
            scaled_maxp,
        );
        Ok(path.join(fname))
    }
}

#[derive(Debug, Clone)]
pub enum DpmTestMode {
    LocalWithOneMat,
    SemiGlobal,
}
impl DpmTestMode {
    fn get_tag(&self) -> &str {
        match self {
            DpmTestMode::LocalWithOneMat => "local_with_one_mat",
            DpmTestMode::SemiGlobal => "semi_global",
        }
    }
    fn from_tag(tag: &str) -> Result<Self> {
        match tag {
            "local_with_one_mat" => Ok(DpmTestMode::LocalWithOneMat),
            "semi_global" => Ok(DpmTestMode::SemiGlobal),
            _ => bail!("Unknown tag: {}", tag),
        }
    }
}
