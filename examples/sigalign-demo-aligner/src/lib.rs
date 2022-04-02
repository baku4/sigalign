use anyhow::{Result, format_err, bail as error_msg};

use clap::{
    App,
    AppSettings,
};

mod reference;
mod alignment;

use reference::{
    ReferenceConfig,
    ReferencePaths,
    SelfDescReference,
};
use alignment::AlignmentConfig;

pub struct Application;

impl Application {
    pub fn run() {
        let matches = App::new("sigalign-demo-aligner")
            .version("0.1.0")
            .author("baku <bahkhun@gmail.com>")
            .about("Binary demo implementation")
            .global_setting(AppSettings::DeriveDisplayOrder)
            .global_setting(AppSettings::PropagateVersion)
            .global_setting(AppSettings::ArgRequiredElseHelp)
            .subcommand(
                ReferenceConfig::add_args(
                    App::new("reference")
                )
            )
            .subcommand(
                AlignmentConfig::add_args(
                    App::new("alignment")
                )
            )
            .get_matches();
        
        match matches.subcommand() {
            Some(("reference", sub_matches)) => {
                ReferenceConfig::run_command(sub_matches)
            },
            Some(("alignment", sub_matches)) => {
                AlignmentConfig::run_command(sub_matches)
            },
            _ => unreachable!(""),
        }
    }
}