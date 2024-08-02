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
use super::{
    write_alignment_result_as_tsv, ForwardDirection, ReverseDirection,
};

use sigalign::{
    algorithms::Local,
    Aligner,
    Reference,
};
use sigalign_utils::{
    sequence_reader::{
        fasta::FastaReader,
        SeqRecord as _, IdRefRecord as _,
    },
    sequence_manipulation::reverse_complementary::reverse_complement_of_dna_sequence_in_place,
};

type DefaultAligner = Aligner<Local>;

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

        Ok(
            Self {
                input_fasta_file,
                reference_path,
                px,
                po,
                pe,
                minl,
                maxp,
            }
        )
    }
    fn perform_alignment(&self) -> Result<()> {
        // Preparing
        //  - Output
        let stdout = std::io::stdout();
        let lock = stdout.lock();
        let mut buf_writer = std::io::BufWriter::with_capacity(
            32 * 1024,
            lock,
        );
        let mut itoa_buffer = itoa::Buffer::new();
        //  - Aligner
        let mut aligner = self.make_aligner()?;
        let mut query = Vec::new();
        //  - Reference
        let reference_chunk_path = self.reference_path.load_reference_chunk_paths()?;

        // Perform alignment
        for (reference_index, chunk_path) in reference_chunk_path.into_iter().enumerate() {
            eprintln!("Load reference chunk {}: {:?}", reference_index, chunk_path);
            // 1) Prepare reference
            let reference_file = File::open(chunk_path)?;
            let reference = Reference::load_from(reference_file)?;

            let query_file = File::open(&self.input_fasta_file)?;
            let mut fasta_reader = FastaReader::new(query_file);

            while let Some(mut record) = fasta_reader.next() {
                // Forward
                query.clear();
                record.extend_seq_buf(&mut query);
                let result = aligner.align(
                    &query,
                    &reference,
                );
                write_alignment_result_as_tsv::<ForwardDirection>(
                    result,
                    &mut buf_writer,
                    &mut itoa_buffer,
                    &reference_index,
                    record.id(),
                );

                // Reverse complement
                reverse_complement_of_dna_sequence_in_place(&mut query);
                let result = aligner.align(
                    &query,
                    &reference,
                );
                write_alignment_result_as_tsv::<ReverseDirection>(
                    result,
                    &mut buf_writer,
                    &mut itoa_buffer,
                    &reference_index,
                    record.id(),
                );
            }
        }

        Ok(())
    }
    fn make_aligner(&self) -> Result<DefaultAligner> {
        let algorithm = Local::new(
            self.px,
            self.po,
            self.pe,
            self.minl,
            self.maxp,
        )?;
        let aligner = DefaultAligner::new(algorithm);

        Ok(aligner)
    }
}
