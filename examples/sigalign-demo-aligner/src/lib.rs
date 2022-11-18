use anyhow::{Result, format_err, bail as error_msg};

use clap::{
    Command,
    ColorChoice,
};

mod reference;
use reference::{
    ReferenceApp,
    Reference,
    ReferencePaths,
    InnerReference,
};

#[cfg(not(any(feature = "tsv", feature = "thread")))]
mod default_alignment;
#[cfg(not(any(feature = "tsv", feature = "thread")))]
use default_alignment::AlignmentApp;

#[cfg(feature = "tsv")]
mod tsv_alignment;
#[cfg(feature = "tsv")]
use tsv_alignment::AlignmentApp;

#[cfg(feature = "thread")]
mod pool_alignment;
#[cfg(feature = "thread")]
use pool_alignment::AlignmentApp;

pub struct Application;

impl Application {
    pub fn run() {
        let app = Command::new("sigalign-demo-aligner")
            .version("0.1.0")
            .author("baku <bahkhun@gmail.com>")
            .about("Binary demo implementation")
            .arg_required_else_help(true)
            .propagate_version(true)
            .subcommand_required(true)
            .subcommand(ReferenceApp::get_command().display_order(1))
            .subcommand(AlignmentApp::get_command().display_order(2));
        
        let matches = app.get_matches();
        
        match matches.subcommand() {
            Some(("reference", sub_matches)) => {
                ReferenceApp::run(sub_matches)
            },
            Some(("alignment", sub_matches)) => {
                AlignmentApp::run(sub_matches)
            },
            _ => unreachable!(""),
        }
    }
}
