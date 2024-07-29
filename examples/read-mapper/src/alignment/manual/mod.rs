use std::path::PathBuf;

use crate::{
    Result, error, error_msg,
    reference::ReferencePathDetector,
};
use super::arg_parser::{
    check_input_file_extension_is_allowed,
};
use clap::{arg, value_parser, Arg, ArgMatches, Command};

pub struct ManualAlignmentApp;

#[derive(Debug, Clone)]
struct Config {
    // Input
    input_file: PathBuf,
    is_gzip_compressed: bool,
    is_fasta_file: bool, // true: FASTA, false: FASTQ
    reference_path: ReferencePathDetector,
    // Alignment Regulator
    px: u32,
    po: u32,
    pe: u32,
    minl: u32,
    maxp: f32,
    // Algorithm
    is_local: bool,
    limit: Option<u32>,
    chunk: Option<(u32, u32)>,
    // Threads
    num_threads: usize,
    // Output Format
    output_is_sam: bool, // true: SAM, false: TSV
}

impl ManualAlignmentApp {
    pub fn tag() -> &'static str {
        "manual"
    }
    pub fn get_command() -> Command {
        Command::new(Self::tag())
            .about("Align reads to reference with manually defined parameters")
            .arg_required_else_help(true)
            .arg(arg!(-i --input <FILE> "Input query FASTA or FASTQ file path (can be gzipped)")
                .value_parser(value_parser!(PathBuf))
                .required(true)
                .display_order(1))
            .arg(arg!(-r --reference <FILE> "SigAlign reference file generated by 'reference' subcommand")
                .value_parser(value_parser!(PathBuf))
                .required(true)
                .display_order(2))
            .arg(Arg::new("penalties").short('p').long("penalties")
                .value_names(["INT", "INT", "INT"])
                .num_args(3)
                .help("Mismatch, Gap-open and Gap-extend penalties")
                .required(true)
                .display_order(3))
            .arg(Arg::new("cutoffs").short('c').long("cutoffs")
                .value_names(["INT", "FLOAT"])
                .num_args(2)
                .help("Minimum length and Maximum penalty per length")
                .required(true)
                .display_order(4))
            .arg(Arg::new("limit").short('l').long("limit")
                .help("Limit the number of alignments for each query")
                .value_parser(value_parser!(u32))
                .required(false)
                .display_order(5))
            .arg(Arg::new("chunk").short('u').long("chunk")
                .help("Perform chunked alignment with (chunk size, sliding size)")
                .value_names(["INT", "INT"])
                .value_parser(value_parser!(u32))
                .required(false)
                .display_order(6))
            .arg(Arg::new("thread").short('t').long("thread")
                .help("The number of threads")
                .default_value("4")
                .value_parser(value_parser!(u32))
                .required(false)
                .display_order(7))
            .arg(arg!(--semi_global "Use semi-global alignment instead of local")
                .display_order(8)
                .required(false))
            .arg(arg!(--tsv "Output format is TSV instead of SAM")
                .display_order(9)
                .required(false))
    }
    pub fn run(matches: &ArgMatches) -> Result<()> {
        let config = Config::from_matches(matches)?;
        println!("{:#?}", config);
        todo!()
    }
}

impl Config {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        // Input file paths
        let input_file = matches.get_one::<PathBuf>("input").unwrap().clone();
        let (is_gzip_compressed, is_fasta_file) = check_input_file_extension_is_allowed(&input_file)?;
        
        let reference_path = {
            let path = matches.get_one::<PathBuf>("reference").ok_or(error!("Invalid reference fasta"))?.clone();
            // if !path.exists() {
            //     error_msg!("Reference file does not exist: {:?}", path);
            // } FIXME: Revive later
            ReferencePathDetector::new(&path)
        };

        // Alignment Regulator
        let (px, po, pe) = {
            let mut iterator: clap::parser::ValuesRef<_> = matches.get_many::<String>("penalties").unwrap();
            let px: u32 = iterator.next().unwrap().parse().map_err(
                |_| error!("Mismatch penalty allows only positive integer")
            )?;
            let po: u32 = iterator.next().unwrap().parse().map_err(
                |_| error!("Gap-open penalty allows only non-negative integer")
            )?;
            let pe: u32 = iterator.next().unwrap().parse().map_err(
                |_| error!("Gap-extend penalty allows only positive integer")
            )?;
            (px, po, pe)
        };
        let (minl, maxp) = {
            let mut iterator: clap::parser::ValuesRef<_> = matches.get_many::<String>("cutoffs").unwrap();
            let minl: u32 = iterator.next().unwrap().parse().map_err(
                |_| error!("Minimum length allows only positive integer")
            )?;
            let maxp: f32 = iterator.next().unwrap().parse().map_err(
                |_| error!("Maximum penalty per length allows only positive float")
            )?;
            (minl, maxp)
        };

        // Algorithm
        let is_local = if matches.get_flag("semi_global") {
            false
        } else {
            true
        };
        let limit = matches.get_one::<u32>("limit").copied();
        let chunk = matches.get_one::<(u32, u32)>("chunk").copied();

        // Others
        let num_threads = matches.get_one::<u32>("thread").copied().unwrap() as usize;
        let output_is_sam = if matches.get_flag("tsv") {
            false
        } else {
            true
        };

        Ok(Self {
            input_file,
            is_gzip_compressed,
            is_fasta_file,
            reference_path,
            px,
            po,
            pe,
            minl,
            maxp,
            is_local,
            limit,
            chunk,
            num_threads,
            output_is_sam,
        })
    }
}
