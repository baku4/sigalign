use std::{
    path::PathBuf,
    time::Instant,
};
use clap::{
    Command,
    arg,
    ArgMatches,
    value_parser,
};
use log::{info, debug};

use crate::core::{Result, error_msg};
use super::ReferencePaths;

mod wrapper;
pub use wrapper::{
    SigReferenceWrapper,
    InnerReference,
};

pub struct ReferenceApp;
#[derive(Debug, Clone)]
struct ReferenceConfig {
    // Input file path(s)
    input_files: Vec<PathBuf>,
    output_file_stem: PathBuf,
    // Configuration for SigAlign reference
    max_len_reference: usize,
    klt_byte_size: usize,
    sa_sampling_ratio: u64,
}

impl ReferenceApp {
    pub fn get_command() -> Command {
        debug!("ReferenceApp::get_command");
        Command::new("reference")
            .about("Generate reference file")
            .arg_required_else_help(true)
            // Required
            .arg(arg!(-i --input <FILE> "Input FASTA path(s)")
                .display_order(1)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-o --output <FILE> "Output reference path's stem")
                .display_order(2)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            // Optional
            .arg(arg!(-w --overwrite  "Overwrite output reference file")
                .display_order(3)
                .value_parser(value_parser!(bool))
                .required(false))
            .arg(arg!(-l --maxlen <INT> "Maximum length per each reference")
                .display_order(8)
                .value_parser(value_parser!(usize))
                .required(false))
            .arg(arg!(-k --maxklt <INT> "Maximum k-mer lookup table size (bytes)")
                .display_order(8)
                .value_parser(value_parser!(usize))
                .required(false))
            .arg(arg!(-s --sasr <INT>  "Suffix array sampling ratio")
                .display_order(4)
                .value_parser(value_parser!(u64))
                .required(false))
    }
    pub fn run(matches: &ArgMatches) {
        debug!("ReferenceApp::run");
        let total_start = Instant::now();
        
        let config = {
            eprintln!("Parsing configuration");
            let start = Instant::now();
            let config = ReferenceConfig::from_matches(matches).unwrap();
            eprintln!("{:#?}", config);
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());
            config
        };

        {
            eprintln!("# 2. Build and save reference");
            let start = Instant::now();
            config.build_and_save_references().unwrap();
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());
        };
        
        eprintln!("# 4. All processes are completed");
        eprintln!(" - Total time elapsed: {} s", total_start.elapsed().as_secs_f64());
    }
}

impl ReferenceConfig {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        debug!("ReferenceConfig::from_matches");
        // (1) Path
        let input_files = {
            let mut input_files = Vec::<PathBuf>::new();
            let mut iterator = matches.get_many::<PathBuf>("input").expect("Invalid input file paths");
            iterator.for_each(|path| {
                match path.extension().expect("") {

                }
            })
        };

        let (px, po, pe) = {
            let mut iterator: clap::parser::ValuesRef<_> = matches.get_many::<String>("penalties").unwrap();
            let px: u32 = iterator.next().unwrap().parse().expect("Mismatch penalty allows integer.");
            let po: u32 = iterator.next().unwrap().parse().expect("Gap-open penalty allows integer.");
            let pe: u32 = iterator.next().unwrap().parse().expect("Gap-extend penalty allows integer.");

            (px, po, pe)
        };


        let overwrite = matches.get_flag("overwrite");
        let output_file_pathbuf = {
            matches.get_one::<PathBuf>("output").expect("Invalid output file").clone()
        };
        if !overwrite && output_file_pathbuf.exists() {
            error_msg!("Output file already exist")
        }

        // (2) Sequence storage type
        let divide_size = match matches.get_one::<usize>("divide") {
            Some(v) => Some(*v),
            None => None,
        };
        let use_rc = matches.get_flag("reverse");
        
        // (3) Pattern finder config
        let kmer = match matches.get_one::<usize>("klt") {
            Some(v) => Some(*v),
            None => None,
        };
        let sa_sampling_ratio = match matches.get_one::<u64>("ssr") {
            Some(v) => Some(*v),
            None => None,
        };

        Ok(
            Self {
                input_files: input_file_pathbuf,
                output_file_stem: output_file_pathbuf,
                max_len_reference: divide_size,
                use_rc,
                kmer,
                sa_sampling_ratio,
            }
        )
    }
    fn build_and_save_references(&self) -> Result<()> {
        let divided_sequence_storages = {
            let sss = SigReferenceWrapper::get_divided_sequence_storages(
                &self.input_files,
                self.use_rc,
                self.max_len_reference,
            )?;
            eprintln!(" Storage is divided into {}.", sss.len());
            sss
        };

        let reference_paths = ReferencePaths::new_to_save(
            &self.output_file_stem,
            divided_sequence_storages.len(),
        );

        divided_sequence_storages.into_iter().enumerate().zip(reference_paths.0).for_each(|((idx, ss), file_path)| {
            eprint!(" Ref idx {}; ", idx);
            SigReferenceWrapper::build_and_save(
                ss,
                self.kmer,
                self.sa_sampling_ratio,
                &file_path,
            ).unwrap();
        });

        Ok(())
    }
}
