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
    Reference,
    InnerReference,
};

pub struct ReferenceApp;
#[derive(Debug, Clone)]
struct ReferenceConfig {
    // Path
    input_file_pathbuf: PathBuf,
    output_file_pathbuf: PathBuf,
    // Pattern finder config
    use_128_bwt: bool,
    kmer: Option<usize>,
    sa_sampling_ratio: Option<u64>,
}

impl ReferenceApp {
    pub fn get_command() -> Command {
        Command::new("reference")
            .about("Generate reference file (Sequence storage is indexed fasta (with .fai))")
            .arg_required_else_help(true)
            .arg(arg!(-i --input <FILE> "Input FASTA path").display_order(1)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-o --output <FILE> "Output reference path").display_order(2)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-w --overwrite  "Overwrite output reference file").display_order(3))
            .arg(arg!(-c --cpb  "Use higher compressed (128) Bwt block").display_order(5))
            .arg(arg!(-s --ssr <INT>  "Suffix array sampling ratio").display_order(6)
                .value_parser(value_parser!(u64))
                .required(false))
            .arg(arg!(-k --klt <INT> "Kmer size for count array lookup table").display_order(7)
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

        let references = {
            eprintln!("# 2. Build reference");
            let start = Instant::now();
            let references =  config.build_references().unwrap();
            eprintln!("  {} separated references are built.", references.len());
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());

            references
        };

        {
            eprintln!("# 3. Save reference");
            let start = Instant::now();
            config.save_references(references);
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());
        }
        
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
        // Unsupported
        
        // (3) Pattern finder config
        let use_128_bwt = matches.get_flag("cpb");
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
                use_128_bwt,
                kmer,
                sa_sampling_ratio,
            }
        )
    }
    fn build_references(&self) -> Result<Vec<Reference>> {
        Reference::build(
            &self.input_file_pathbuf,
            self.use_128_bwt,
            self.kmer,
            self.sa_sampling_ratio,
        )
    }
    fn save_references(&self, references: Vec<Reference>) {
        let reference_count = references.len();
        let reference_paths = ReferencePaths::new_to_save(
            &self.output_file_pathbuf,
            reference_count,
        );

        references.into_iter().enumerate().zip(reference_paths.0).for_each(|((idx, reference), file_path)| {
            let estimated_size = reference.size_of();
            eprintln!(" Ref idx {} was saved; about {} bytes", idx, estimated_size);
            reference.save_to_file(&file_path).unwrap();
        });
    }
}
