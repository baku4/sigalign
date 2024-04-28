use anyhow::{Result, bail as error_msg, anyhow as error};

mod reference;
use clap::Command;
use reference::ReferenceApp;
mod alignment;
use alignment::AlignmentApp;

pub struct Application;

impl Application {
    pub fn run() -> Result<()> {
        let cmd = Command::new("sig-read-mapper")
            .bin_name("sig-read-mapper")
            .version("0.1.0")
            .author("baku <bahkhun@gmail.com>")
            .about("Binary demo implementation")
            .arg_required_else_help(true)
            .propagate_version(true)
            .subcommand_required(true)
            .subcommand(ReferenceApp::get_command().display_order(1))
            .subcommand(AlignmentApp::get_command().display_order(2));
        
        let matches = cmd.get_matches();
        
        match matches.subcommand() {
            Some(("reference", sub_matches)) => {
                ReferenceApp::run(sub_matches)?;
            },
            Some(("alignment", sub_matches)) => {
                AlignmentApp::run(sub_matches)?;
            },
            _ => unreachable!(""),
        }

        Ok(())
    }
}
