use std::{fs::File, io::{BufRead as _, BufReader, Write}};
use anyhow::Result;

use sigalign::{results::LabeledQueryAlignment, utils::formatter::SamFormatter};

mod tsv;
use tsv::TsvFormatter;

use crate::reference::ReferencePathDetector;

pub enum ResFormatter {
    TSV(TsvFormatter),
    SAM(SamFormatter),
}

impl ResFormatter {
    pub fn new(is_sam: bool) -> Self {
        if is_sam {
            Self::SAM(SamFormatter::new())
        } else {
            Self::TSV(TsvFormatter::new())
        }
    }
    pub fn write_header(
        &mut self,
        writer: &mut impl Write,
        reference_path_detector: &ReferencePathDetector,
    ) -> Result<()> {
        match self {
            Self::TSV(formatter) => formatter.write_header(writer),
            Self::SAM(formatter) => {
                formatter.write_hd_header(writer)?;
                // Fields of manifest file:
                // file_index, file_name, record_index, reference_index, target_index, target_label, target_length
                let manifest_file = File::open(reference_path_detector.get_manifest_file_path())?;
                let reader = BufReader::new(manifest_file);
                for line in reader.lines().skip(1) {
                    let line = line?;
                    let fields: Vec<&str> = line.split('\t').collect();
                    formatter.write_sq_header(
                        writer,
                        &fields[5],
                        &fields[6].parse::<u32>().unwrap(),
                    )?;
                }
                Ok(())
            },
        }
    }
    pub fn write_record(
        &mut self,
        writer: &mut impl Write,
        labeled_query_alignment: &LabeledQueryAlignment,
        query_name: &str,
        query_length: u32,
        is_forward: bool,
    ) -> Result<()> {
        match self {
            Self::TSV(formatter) => formatter.write_tsv_record(
                writer, labeled_query_alignment, query_name, is_forward,
            ),
            Self::SAM(formatter) => {
                formatter.write_labeled_query_alignment_with_hclip(
                    writer, labeled_query_alignment, query_name, is_forward, query_length,
                )?;
                Ok(())
            }
        }
    }
}
