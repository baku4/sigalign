use crate::Result;
use clap::{ArgMatches, Command};

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
        let _tag_1 = ManualAlignmentApp::tag();
        match matches.subcommand() {
            Some((_tag_1, sub_matches)) => ManualAlignmentApp::run(sub_matches),
            _ => unreachable!(),
        }
    }
}
