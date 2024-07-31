use crate::Result;

use std::io::Write;

#[inline]
pub fn write_manifest_header<W: Write>(mut writer: W) -> Result<()> {
    writer.write(b"file_index\tfile_name\trecord_index\treference_index\ttarget_index\ttarget_label\ttarget_length\n")?;
    Ok(())
}

#[inline]
pub fn write_manifest_line<W: Write>(
    mut writer: W,
    file_index: &usize,
    file_name: &str,
    record_index: &usize,
    reference_index: &usize,
    target_index: &u32,
    target_label: &str,
    target_length: &usize,
) -> Result<()> {
    writer.write(
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            file_index, file_name, record_index, reference_index, target_index, target_label, target_length
        )
        .as_bytes(),
    )?;
    Ok(())
}
