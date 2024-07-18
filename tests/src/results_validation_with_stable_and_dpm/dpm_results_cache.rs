use std::{io::{Read, Write}, path::{Path, PathBuf}};

use log::info;
use sigalign::results::TargetAlignment;

use crate::common::{
    directory_path::get_target_dir,
    test_data::DataForValidation,
    configuration::TestSetting,
    dynamic_programming_matrix::{
        dp_local_with_one_mat_to_target,
        dp_local_with_all_subs_to_target,
        dp_semi_global_to_target,
    },
};

const CACHE_DIR_NAME: &str = "dpm_results_cache_failed_in_stable";
const PREC_SCALE_FOR_MAXP: u32 = 10_000;

pub struct DpmAlignerWithCache {
    mode: DpmMode,
    test_data_tag: String,
    query_index: u32,
    target_index: u32,
    px: u32,
    po: u32,
    pe: u32,
    minl: u32,
    maxp: f32,
    cache_file_path: PathBuf,
}
#[derive(Debug, Clone)]
pub enum DpmMode {
    SemiGlobal,
    LocalWithOneMat,
    LocalWithAllSubs,
}
impl DpmMode {
    fn to_str(&self) -> &str {
        match self {
            DpmMode::SemiGlobal => "semi_global",
            DpmMode::LocalWithOneMat => "local_with_one_mat",
            DpmMode::LocalWithAllSubs => "local_with_all_subs",
        }
    }

}

impl DpmAlignerWithCache {
    pub fn new(
        mode: DpmMode,
        test_data_tag: String,
        query_index: u32, target_index: u32,
        px: u32, po: u32, pe: u32, minl: u32, maxp: f32,
    ) -> Self {
        let mut tmp = DpmAlignerWithCache {
            mode,
            test_data_tag, query_index, target_index,
            px, po, pe, minl, maxp,
            cache_file_path: PathBuf::new(),
        };
        let path = tmp.cache_dir();
        tmp.cache_file_path = path.join(tmp.cache_file_name());
        tmp
    }
    fn cache_dir(&self) -> PathBuf {
        let mut path = get_target_dir().unwrap();
        path.push(CACHE_DIR_NAME);
        if !path.exists() {
            std::fs::create_dir_all(&path).unwrap();
        }
        path
    }
    fn cache_file_name(&self) -> String {
        let scaled_maxp = (self.maxp * PREC_SCALE_FOR_MAXP as f32) as u32;
        format!(
            "{}-{}-{}-{}-{}-{}-{}-{}-{}.json",
            self.mode.to_str(),
            self.test_data_tag,
            self.query_index, self.target_index,
            self.px, self.po, self.pe,
            self.minl, scaled_maxp,
        )
    }

    pub fn perform_alignment_if_needed(
        &self,
        query: &[u8],
        target: &[u8],
    ) -> TargetAlignment {
        let target_alignment = self.load_cahce_if_exist();
        if target_alignment.is_none() {
            info!("DPM cache not found. Performing alignment for query index: {}, target index: {}", self.query_index, self.target_index);
            let target_alignment = self.perform_alignment(query, target);
            self.save_results_to_cache(&target_alignment);
            target_alignment
        } else {
            info!("DPM cache found. Using cache for query index: {}, target index: {}", self.query_index, self.target_index);
            target_alignment.unwrap()
        }
    }
    fn load_cahce_if_exist(&self) -> Option<TargetAlignment> {
        if self.cache_file_path.exists() {
            let mut file = std::fs::File::open(&self.cache_file_path).unwrap();
            let mut data: String = String::new();
            file.read_to_string(&mut data).unwrap();

            let target_alignment = TargetAlignment::from_json(&data).unwrap();
            Some(target_alignment)
        } else {
            None
        }
    }
    fn perform_alignment(
        &self,
        query: &[u8],
        target: &[u8],
    ) -> TargetAlignment {
        let alignments = match self.mode {
            DpmMode::SemiGlobal => {
                dp_semi_global_to_target(
                    query, target,
                    self.px, self.po, self.pe,
                    self.minl, self.maxp,
                )
            },
            DpmMode::LocalWithOneMat => {
                dp_local_with_one_mat_to_target(
                    query, target,
                    self.px, self.po, self.pe,
                    self.minl, self.maxp,
                )
            },
            DpmMode::LocalWithAllSubs => {
                dp_local_with_all_subs_to_target(
                    query, target,
                    self.px, self.po, self.pe,
                    self.minl, self.maxp,
                )
            },
        };
        TargetAlignment {
            index: self.target_index,
            alignments,
        }
    }
    fn save_results_to_cache(&self, target_alignment: &TargetAlignment) {
        let data = target_alignment.to_json();
        let mut file = std::fs::File::create(&self.cache_file_path).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    }
}