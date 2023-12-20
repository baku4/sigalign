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
use sigalign::{
    Reference,
    Aligner,
};

mod perform_alignment_with_pool;
use perform_alignment_with_pool::ThreadPool;

const BATCH_SIZE: usize = 16;
const CHANNEL_CAP_PER_THREAD: usize = 8;

pub struct AlignmentApp;
#[derive(Debug, Clone)]
pub struct AlignmentConfig {
    // Path
    input_fasta_file: PathBuf,
    reference_path: ReferencePathDetector,
    // Condition
    px: u32,
    po: u32,
    pe: u32,
    minl: u32,
    maxp: f32,
    is_local: bool,
    limit: Option<u32>,
    // Worker threads
    num_workers: usize,
}

impl AlignmentApp {
    pub fn get_command() -> Command {
        Command::new("alignment")
            .about("Alignment with FASTA file (use thread pool)")
            .arg_required_else_help(true)
            .arg(arg!(-i --input <FILE> "Input query FASTA path")
                .display_order(1)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-r --reference <FILE> "SigAlign reference file")
                .display_order(2)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(Arg::new("penalties").short('p').long("penalties")
                .display_order(3)
                .value_names(["INT", "INT", "INT"])
                .num_args(3)
                .help("Mismatch, Gap-open and Gap-extend penalties")
                .required(true))
            .arg(Arg::new("cutoffs").short('c').long("cutoffs")
                .display_order(4)
                .value_names(["INT", "FLOAT"])
                .num_args(2)
                .help("Minimum aligned length and maximum penalty per length")
                .required(true))
            .arg(arg!(-l --limit <INT> "Limit of the number of alignments for each query")
                .display_order(5)
                .value_parser(value_parser!(u32))
                .required(false))
            .arg(arg!(-s --semi_global "Use semi-global alignment")
                .display_order(6)
                .required(false))
            .arg(arg!(-t --thread <INT> "The number of thread")
                .display_order(7)
                .value_parser(value_parser!(usize))
                .required(false))
    }
    pub fn run(matches: &ArgMatches) -> Result<()> {
        let total_start = Instant::now();

        // Parse configuration
        let config = {
            let start = Instant::now();
            let config = AlignmentConfig::from_matches(matches)?;
            eprintln!("Configuration parsed ({} s)", start.elapsed().as_secs_f64());
            eprintln!("{:#?}", config);
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
        let input_fasta_file = {
            matches.get_one::<PathBuf>("input").ok_or(error!("Invalid input fasta"))?.clone()
        };
        let reference_path = {
            let path = matches.get_one::<PathBuf>("reference").ok_or(error!("Invalid reference fasta"))?.clone();
            if !path.exists() {
                error_msg!("Reference file does not exist: {:?}", path);
            }
            ReferencePathDetector::new(&path)
        };

        // (2) Condition
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
                |_| error!("Cutoff of MinL allows only positive integer")
            )?;
            let maxp: f32 = iterator.next().unwrap().parse().map_err(
                |_| error!("Cutoff of MaxP allows only positive float")
            )?;
            (minl, maxp)
        };

        // (3) Thread
        let thread = match matches.get_one::<usize>("thread") {
            Some(v) => *v,
            None => 1,
        };
        // (4) Limit
        let limit = match matches.get_one::<u32>("limit") {
            Some(v) => Some(*v),
            None => None,
        };
        // (5) Is local mode
        let is_local = if matches.get_flag("use_semi_global") {
            false
        } else {
            true
        };

        Ok(
            Self {
                input_fasta_file,
                reference_path,
                px,
                po,
                pe,
                minl,
                maxp,
                limit,
                is_local,
                num_workers: thread,
            }
        )
    }
    fn perform_alignment(&self) -> Result<()> {
        let aligner = self.make_aligner()?;
        
        eprintln!("Generate thread pool with {} workers", self.num_workers);
        let mut thread_pool = ThreadPool::new(
            self.num_workers,
            BATCH_SIZE,
            CHANNEL_CAP_PER_THREAD * self.num_workers,
            &self.input_fasta_file,
            aligner,
        );

        let reference_chunk_path = self.reference_path.load_reference_chunk_paths()?;

        for (reference_index, chunk_path) in reference_chunk_path.into_iter().enumerate() {
            eprintln!("Load reference chunk {}: {:?}", reference_index, chunk_path);
            let file = File::open(chunk_path)?;
            let reference = Reference::load_from(file)?;

            eprintln!("Execute alignment with thread pool");
            thread_pool.execute(
                reference,
                reference_index as u32,
            )?;
        }

        thread_pool.clear();
        Ok(())
    }
    fn make_aligner(&self) -> Result<Aligner> {
        let mut aligner = Aligner::new(
            self.px,
            self.po,
            self.pe,
            self.minl,
            self.maxp,
        )?;
        if self.is_local {
            _ = aligner.change_to_local();
        } else {
            _ = aligner.change_to_semi_global();
        }
        if let Some(limit) = self.limit {
            aligner.set_limit(Some(limit));
        }

        Ok(aligner)
    }
}
