use crate::{Result, error_msg};
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

use super::ReferencePaths;

mod wrapper;
pub use wrapper::{
    SigReferenceWrapper,
    InnerReference,
};

pub struct ReferenceApp;
#[derive(Debug, Clone)]
struct ReferenceConfig {
    // Path
    input_file_pathbuf: PathBuf,
    output_file_pathbuf: PathBuf,
    // Sequence storage type
    divide_size: Option<usize>,
    use_rc: bool,
    // Pattern finder config
    kmer: Option<usize>,
    sa_sampling_ratio: Option<u64>,
}

impl ReferenceApp {
    pub fn get_command() -> Command {
        Command::new("reference")
            .about("Generate reference file")
            .arg_required_else_help(true)
            .arg(arg!(-i --input <FILE> "Input FASTA path").display_order(1)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-o --output <FILE> "Output reference path").display_order(2)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-w --overwrite  "Overwrite output reference file").display_order(3))
            .arg(arg!(-r --reverse  "Use reverse complementary sequence (for nucleotide)").display_order(4))
            .arg(arg!(-s --ssr <INT>  "Suffix array sampling ratio").display_order(6)
                .value_parser(value_parser!(u64))
                .required(false))
            .arg(arg!(-k --klt <INT> "Kmer size for count array lookup table").display_order(7)
                .value_parser(value_parser!(usize))
                .required(false))
            .arg(arg!(-d --divide <INT> "Split by sequence length").display_order(8)
                .value_parser(value_parser!(usize))
                .required(false))
    }
    pub fn run(matches: &ArgMatches) {
        let total_start = Instant::now();

        let config = {
            eprintln!("# 1. Parsing configuration");
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
        // (1) Path
        let input_file_pathbuf = {
            matches.get_one::<PathBuf>("input").expect("Invalid input file").clone()
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
                input_file_pathbuf,
                output_file_pathbuf,
                divide_size,
                use_rc,
                kmer,
                sa_sampling_ratio,
            }
        )
    }
    fn build_and_save_references(&self) -> Result<()> {
        let divided_sequence_storages = {
            let sss = SigReferenceWrapper::get_divided_sequence_storages(
                &self.input_file_pathbuf,
                self.use_rc,
                self.divide_size,
            )?;
            eprintln!(" Storage is divided into {}.", sss.len());
            sss
        };

        let reference_paths = ReferencePaths::new_to_save(
            &self.output_file_pathbuf,
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
