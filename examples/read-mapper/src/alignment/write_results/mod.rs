use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use anyhow::Ok;

use crate::{Result, error, error_msg};
use crate::reference::ReferencePathDetector;

pub fn write_sam_header<W: Write>(
    mut writer: W,
    reference_path_detector: &ReferencePathDetector,
) -> Result<()> {
    writer.write_all(b"@HD\tVN:1.6\tSO:unsorted\n")?;
    
    // Fields of manifest file:
    // file_index, file_name, record_index, reference_index, target_index, target_label, target_length
    let manifest_file = File::open(reference_path_detector.get_manifest_file_path())?;
    let reader = BufReader::new(manifest_file);
    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split('\t').collect();
        
        writer.write_all(format!("@SQ\tSN:{}\tLN:{}\n", fields[5], fields[6]).as_bytes())?;
    }

    Ok(())
}
