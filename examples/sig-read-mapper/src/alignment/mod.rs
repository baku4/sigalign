use std::{
    path::PathBuf,
    time::Instant, fs::File,
};
use clap::{
    builder::{Command, Arg},
    arg,
    ArgMatches,
    value_parser,
};

use crate::{
    Result, error_msg, error,
    reference::ReferencePathDetector,
};

use sigalign_core::aligner::{
    LocalAligner,
    Aligner, AlignmentRegulator,
};
use sigalign_impl::allocation_strategy::LinearStrategy;
use sigalign_utils::{
    sequence_reader::{
        fasta::FastaReader,
        fastq::FastqReader,
        SeqRecord as _, IdRefRecord as _,
    },
    sequence_manipulation::reverse_complementary::reverse_complement_of_dna_sequence_in_place,
};
use sigalign::Reference;

mod executor;
mod thread;
mod write_results;

type DefaultAligner = LocalAligner<LinearStrategy>;

pub struct AlignmentApp;
#[derive(Debug, Clone)]
pub struct AlignmentConfig {
    // Path
    input_file: PathBuf,
    is_fasta: bool, // true: FASTA, false: FASTQ
    reference_path: ReferencePathDetector,
    // Alignment Regulator
    lambda: f32,
    px: u32,
    po: u32,
    pe: u32,
    // Algorithm
    is_local: bool,
    max: Option<u32>,
    // Threads
    num_threads: usize,
    // Output Format
    is_sam_format: bool, // true: SAM, false: TSV
}

impl AlignmentApp {
    pub fn get_command() -> Command {
        Command::new("alignment")
            .about("Alignment with FASTA/FASTQ file")
            .arg_required_else_help(true)
            .arg(arg!(-i --input <FILE> "Input query FASTA or FASTQ path")
                .value_parser(value_parser!(PathBuf))
                .required(true)
                .display_order(1))
            .arg(arg!(-r --reference <FILE> "SigAlign reference file")
                .value_parser(value_parser!(PathBuf))
                .required(true)
                .display_order(2))
            .arg(Arg::new("lambda").short('l').long("lambda")
                .value_parser(value_parser!(PathBuf))
                .help("Throughput determination parameter")
                .default_value("4")
                .value_parser(value_parser!(f32))
                .required(false)
                .display_order(3))
            .arg(Arg::new("penalties").short('p').long("penalties")
                .value_names(["INT", "INT", "INT"])
                .num_args(3)
                .help("Mismatch, Gap-open and Gap-extend penalties")
                .default_values(["4", "6", "2"])
                .required(false)
                .display_order(4))
            .arg(Arg::new("threads").short('t').long("threads")
                .help("The number of worker threads")
                .default_value("4")
                .value_parser(value_parser!(u32))
                .required(false)
                .display_order(5))
            .arg(Arg::new("max").short('m').long("max")
                .help("Maximum number of alignments for each query (0 for inf)")
                .default_value("0")
                .value_parser(value_parser!(u32))
                .required(false)
                .display_order(6))
            .arg(arg!(--semi_global "Use semi-global alignment")
                .display_order(7)
                .required(false))
            .arg(arg!(--tsv "TSV output format")
                .display_order(8)
                .required(false))
    }
    pub fn run(matches: &ArgMatches) -> Result<()> {
        let total_start = Instant::now();

        // Parse configuration
        let config = {
            let start = Instant::now();
            let config = AlignmentConfig::from_matches(matches)?;
            eprintln!("Configuration parsed ({} s)", start.elapsed().as_secs_f64());
            eprintln!("{:#?}", config); // TODO: Gracefully print
            config
        };

        // Perform alignment
        {
            let start = Instant::now();
            config.perform_alignment()?;
            eprintln!("Alignment done ({} s)", start.elapsed().as_secs_f64());
        };
        eprintln!("Total elapsed time: {} s", total_start.elapsed().as_secs_f64());
        Ok(())
    }
}

impl AlignmentConfig {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        // (1) Path
        let (input_file, is_fasta) = {
            let input_file = matches.get_one::<PathBuf>("input").ok_or(error!("Invalid input files"))?.clone();
            // FIXME: Check the types
            let is_fasta = true;
            (input_file, is_fasta)
        };
        let reference_path = {
            let path = matches.get_one::<PathBuf>("reference").ok_or(error!("Invalid reference fasta"))?.clone();
            // if !path.exists() { //FIXME: revive
            //     error_msg!("Reference file does not exist: {:?}", path);
            // }
            ReferencePathDetector::new(&path)
        };

        // (2) Regulator
        let lambda: f32 = {
            match matches.get_one::<f32>("lambda") {
                Some(lambda) => *lambda,
                None => error_msg!("Invalid lambda value"),
            }
        };
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

        // (3) Thread
        let threads = match matches.get_one::<u32>("threads") {
            Some(v) => *v,
            None => 1,
        };
        // (4) Max number of outputs
        let max = match matches.get_one::<u32>("max") {
            Some(v) => {
                if *v == 0 {
                    None
                } else {
                    Some(*v)
                }
            },
            None => None,
        };
        // (5) Is local mode
        let is_local = if matches.get_flag("semi_global") {
            false
        } else {
            true
        };
        // (6) Output format
        let is_sam_format = if matches.get_flag("tsv") {
            false
        } else {
            true
        };

        Ok(
            Self {
                input_file,
                reference_path,
                is_fasta,
                lambda,
                px,
                po,
                pe,
                is_local,
                max,
                num_threads: threads as usize,
                is_sam_format,
            }
        )
    }
    // TODO:
    fn perform_alignment(&self) -> Result<()> {
        // Get offsets of query for each thread
        Ok(())
    }
}
