use std::path::PathBuf;

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
            file_name.push(format!(".{}", index));
            self.original_path.with_file_name(file_name)
        }
    }
    pub fn get_manifest_file_path(&self) -> PathBuf {
        let mut manifest_file_name = self.original_path.file_name().unwrap().to_os_string();
        manifest_file_name.push(".manifest");
        self.original_path.with_file_name(manifest_file_name)
    }
}

// impl ReferencePaths {
//     pub fn new_to_save(
//         ref_file_path: &PathBuf,
//         reference_count: usize
//     ) -> Self {
//         let reference_paths = (0..reference_count).map(|ref_idx| {
//             if ref_idx == 0 {
//                 ref_file_path.clone()
//             } else {
//                 let new_path = ref_file_path.clone();
//                 let mut file_name = ref_file_path.file_name().unwrap().to_os_string();
//                 file_name.push(format!("{}", ref_idx));
//                 new_path.with_file_name(file_name)
//             }
//         }).collect();
        
//         Self(reference_paths)
//     }
//     pub fn new_for_load(
//         ref_file_path: &PathBuf,
//     ) -> Self {
//         let mut reference_paths = vec![ref_file_path.clone()];

//         let mut suffix_number = 1;
//         loop {
//             let mut suffix_added_file_name = ref_file_path.file_name().unwrap().to_os_string();
//             suffix_added_file_name.push(format!("{}", suffix_number));

//             let additional_ref_path = ref_file_path.clone().with_file_name(suffix_added_file_name);
//             if additional_ref_path.exists() {
//                 reference_paths.push(additional_ref_path);
//                 suffix_number += 1;
//             } else {
//                 break;
//             }
//         }

//         Self(reference_paths)
//     }
// }
