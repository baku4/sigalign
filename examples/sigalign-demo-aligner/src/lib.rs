use clap::{
    arg, App,
    AppSettings,
};

use std::path::Path;

pub struct Configuration {

}

impl Configuration {
    pub fn new() {
        let matches = App::new("sigalign-demo-aligner")
            .version("0.1.0")
            .author("baku <bahkhun@gmail.com>")
            .about("Binary demo implementation")
            .global_setting(AppSettings::DeriveDisplayOrder)
            .global_setting(AppSettings::PropagateVersion)
            .global_setting(AppSettings::ArgRequiredElseHelp)
            .subcommand(
                App::new("reference")
                    .about("Generate reference file")
                    .arg(
                        arg!(-i --input <PATH> "Input FASTA path")
                    ).arg(
                        arg!(-o --output <PATH> "Output reference path")
                    ).arg(
                        arg!(-r - -reverse  "Use reverse complementary sequence")
                    ).arg(
                        arg!(-c - -compbwt  "Use higher compressed (128) Bwt block")
                    ).arg(
                        arg!(-s --sasr <INT>  "Suffix array sampling ratio")
                            .required(false)
                    ).arg(
                        arg!(-k --klt <INT> "Kmer size for count array lookup table")
                        .required(false)
                    ).arg(
                        arg!(-n - -nomem  "Use index-fasta provider instead of in-memory")
                    )
            )
            .get_matches();
    }
}