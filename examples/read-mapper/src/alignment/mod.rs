use clap::{arg, value_parser, Arg, ArgMatches, Command};
use std::{fs::File, path::PathBuf, time::Instant};

use crate::{error, error_msg, reference::ReferencePathDetector, Result};

use sigalign::{
    algorithms::{Local, SemiGlobal},
    Aligner, Reference,
};
use sigalign_utils::{
    sequence_manipulation::reverse_complementary::reverse_complement_of_dna_sequence_in_place,
    sequence_reader::{fasta::FastaReader, fastq::FastqReader, IdRefRecord as _, SeqRecord as _},
};

// Common module
mod arg_parser;
mod query_reader;
mod write_results;
// Alignment app
mod manual;
use manual::ManualAlignmentApp;

pub struct AlignmentApp;

impl AlignmentApp {
    pub fn get_command() -> Command {
        Command::new("alignment")
            .about("Align reads to reference")
            .arg_required_else_help(true)
            .propagate_version(true)
            .subcommand_required(true)
            .subcommand(ManualAlignmentApp::get_command().display_order(1))
    }
    pub fn run(matches: &ArgMatches) -> Result<()> {
        let tag_1 = ManualAlignmentApp::tag();
        match matches.subcommand() {
            Some((tag_1, sub_matches)) => ManualAlignmentApp::run(sub_matches),
            _ => unreachable!(),
        }
    }
}
