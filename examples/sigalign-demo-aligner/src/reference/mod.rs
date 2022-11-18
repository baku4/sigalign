use super::{Result, error_msg};

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

mod reference_path;
pub use reference_path::ReferencePaths;
mod in_memory_reference;
pub use in_memory_reference::{
    Reference,
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
    use_128_bwt: bool,
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
            .arg(arg!(-c --cpb  "Use higher compressed (128) Bwt block").display_order(5))
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
        let overwrite = matches.contains_id("overwrite");
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
        let use_rc = matches.contains_id("reverse");
        
        // (3) Pattern finder config
        let use_128_bwt = matches.contains_id("cpb");
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
                use_128_bwt,
                kmer,
                sa_sampling_ratio,
            }
        )
    }
    fn build_references(&self) -> Result<Vec<Reference>> {
        Reference::build(
            &self.input_file_pathbuf,
            self.divide_size,
            self.use_rc,
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
