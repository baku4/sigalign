use std::path::Path;
use std::{fs::read, io::{Read, Write, BufReader}};
use std::fs::File;

// /*  Reference struct  */
// use crate::reference::Reference;

// impl Reference {
//     pub fn write_index_to<W>(&self, writer: W) -> Result<(), String>
//         where W: Write 
//     {
//         match bincode::serialize_into(writer, self) {
//             Ok(_) => Ok(()),
//             Err(err) => {
//                 Err(format!("[bincode error] {}", err))
//             },
//         }
//     }
//     /// Write [FmIndex] to file
//     pub fn write_index_to_file(&self, file_path: &str) -> Result<(), String> {
//         let file = {
//             match File::create(file_path) {
//                 Ok(file) => file,
//                 Err(err) => { return Err(format!("{}", err)); }
//             }
//         };
//         self.write_index_to(file)
//     }
//     /// Read [FmIndex] from reader
//     pub fn read_index_from<R>(reader: R) -> Result<Self, String>
//         where R: Read 
//     {
//         match bincode::deserialize_from::<R, Self>(reader) {
//             Ok(fm_index) => {
//                 Ok(fm_index)
//             },
//             Err(err) => {
//                 Err(format!("[bincode error]{:?}", err))
//             },
//         }
//     }
//     /// Read [FmIndex] from file
//     pub fn read_index_from_file(file_path: &str) -> Result<Self, String> {
//         let file = {
//             match File::open(file_path) {
//                 Ok(file) => file,
//                 Err(err) => { return Err(format!("{}", err)); }
//             }
//         };
//         Self::read_index_from(file)
//     }
// }


/*  Lt Fm Index  */
use lt_fm_index::{LtFmIndex, LtFmIndexBuilder};

const LT_FMI_EXT: &str = "lfi";

pub fn read_lt_fm_index_from_file_path(file_path: &str) -> Result<LtFmIndex, String> {
    let file = std::fs::File::open(file_path).unwrap();

    match LtFmIndex::load_from(file) {
        Ok(v) => Ok(v),
        Err(err) => Err("err".to_string())
    }
}
pub fn read_lt_fm_index_from_inferred_path(org_file_path: &str) -> Result<LtFmIndex, String> {
    let path = Path::new(org_file_path);
    let dir = path.parent().unwrap();
    let file_name = path.file_name().unwrap();
    // TODO: Split once
    let first_stem: String = file_name.to_str().unwrap().to_string().split('.').into_iter().next().unwrap().to_string();
    let fm_index_path = dir.join(format!("{}.{}", first_stem, LT_FMI_EXT)).into_os_string().into_string().unwrap();

    let file = std::fs::File::open(fm_index_path).unwrap();

    match LtFmIndex::load_from(file) {
        Ok(v) => Ok(v),
        Err(err) => Err("err".to_string())
    }
}
