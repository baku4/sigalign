use super::Result;
use std::{
    path::PathBuf,
    time::Instant,
    io::{Write, BufWriter, StdoutLock},
};
use clap::{
    builder::{Command, Arg},
    arg,
    ArgMatches,
    value_parser,
};

use crate::reference::{
    SigReferenceWrapper,
    ReferencePaths,
};
use sigalign::{
    aligner::{
        Aligner,
        mode::LocalMode,
        allocation_strategy::LinearStrategy,
    },
    results::{
        AlignmentResult,
        TargetAlignmentResult,
        AlignmentOperations,
        AlignmentOperation,
    },
    utils::{FastaReader, reverse_complement_of_dna},
};
type SigAligner = Aligner<LocalMode, LinearStrategy>;

pub struct AlignmentApp;
#[derive(Debug, Clone)]
pub struct AlignmentConfig {
    // Path
    input_fasta_pathbuf: PathBuf,
    input_reference_paths: ReferencePaths,
    // Condition
    px: u32,
    po: u32,
    pe: u32,
    min_len: u32,
    max_ppl: f32,
}

impl AlignmentApp {
    pub fn get_command() -> Command {
        Command::new("alignment")
            .about("Alignment with FASTA file (print the result as TSV to stdout)")
            .arg_required_else_help(true)
            .arg(arg!(-i --input <FILE> "Input query FASTA path").display_order(1)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-r --reference <FILE> "SigAlign reference file").display_order(2)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(Arg::new("penalties").short('p').long("penalties")
                .value_names(["INT", "INT", "INT"])
                .num_args(3)
                .help("Mismatch, Gap-open and Gap-extend penalties")
                .required(true)
                .display_order(3))
            .arg(Arg::new("cutoffs").short('c').long("cutoffs")
                .value_names(["INT", "FLOAT"])
                .num_args(2)
                .help("Minimum aligned length and maximum penalty per length")
                .required(true)
                .display_order(4))
    }
    pub fn run(matches: &ArgMatches) {
        let total_start = Instant::now();

        let config = {
            eprintln!("# 1. Parsing configuration");
            let start = Instant::now();
            let config = AlignmentConfig::from_matches(matches).unwrap();
            eprintln!("{:#?}", config);
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());
            config
        };

        let aligner = {
            eprintln!("# 2. Make aligner");
            let start = Instant::now();
            let aligner = config.make_aligner();
            eprintln!("{:#?}", aligner);
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());

            aligner
        };

        {
            eprintln!("# 3. Perform alignment");
            let start = Instant::now();
            config.perform_alignment(aligner);
            eprintln!(" - Time elapsed: {} s\n", start.elapsed().as_secs_f64());
        };
        
        eprintln!("# 4. All processes are completed");
        eprintln!(" - Total time elapsed: {} s", total_start.elapsed().as_secs_f64());
    }
}

impl AlignmentConfig {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        // (1) Path
        let input_fasta_pathbuf = {
            matches.get_one::<PathBuf>("input").expect("Invalid input fasta").clone()
        };
        let input_reference_pathbuf = {
            matches.get_one::<PathBuf>("reference").expect("Invalid reference file").clone()
        };
        let input_reference_paths = ReferencePaths::new_for_load(&input_reference_pathbuf);

        // (2) Condition
        let (px, po, pe) = {
            let mut iterator: clap::parser::ValuesRef<_> = matches.get_many::<String>("penalties").unwrap();
            let px: u32 = iterator.next().unwrap().parse().expect("Mismatch penalty allows integer.");
            let po: u32 = iterator.next().unwrap().parse().expect("Gap-open penalty allows integer.");
            let pe: u32 = iterator.next().unwrap().parse().expect("Gap-extend penalty allows integer.");

            (px, po, pe)
        };
        let (min_len, max_ppl) = {
            let mut iterator: clap::parser::ValuesRef<_> = matches.get_many::<String>("cutoffs").unwrap();
            let min_len: u32 = iterator.next().unwrap().parse().expect("Cutoff of MINLEN allows integer.");
            let max_ppl: f32 = iterator.next().unwrap().parse().expect("Cutoff of MAXPPL allows float.");

            (min_len, max_ppl)
        };
        
        Ok(
            Self {
                input_fasta_pathbuf,
                input_reference_paths,
                px,
                po,
                pe,
                min_len,
                max_ppl,
            }
        )
    }
    fn make_aligner(&self) -> SigAligner {
        SigAligner::new(self.px,
            self.po,
            self.pe,
            self.min_len,
            self.max_ppl,
        ).unwrap()
    }
    // TSV line format:
    // | query label | reference index | record index | penalty | length |
    //  query start position | query end position | record start position | record end position |
    //  string operations |
    fn perform_alignment(&self, mut aligner: SigAligner) {
        let stdout = std::io::stdout();
        let lock = stdout.lock();
        let mut buf_writer = std::io::BufWriter::with_capacity(
            32 * 1024,
            lock,
        );
        let mut itoa_buffer = itoa::Buffer::new();

        self.input_reference_paths.0.iter().enumerate().for_each(|(ref_idx, ref_file_path)| {
            eprintln!("  Reference {}", ref_idx);
            let reference = {
                let start = Instant::now();
                let reference = SigReferenceWrapper::load_from_file(ref_file_path).unwrap();
                eprintln!("   - Load reference {} s", start.elapsed().as_secs_f64());
                reference.inner
            };

            // Alignment
            let start = Instant::now();
            let mut sequence_buffer = reference.get_sequence_buffer();
            let fasta_reader = FastaReader::from_path(&self.input_fasta_pathbuf).unwrap();
            fasta_reader.for_each(|(label, query)| {
                // (1) Original Query
                {
                    let result = aligner.alignment(&reference, &mut sequence_buffer, &query);
                    write_alignment_result_as_tsv::<ForwardDirection>(
                        result,
                        &mut buf_writer,
                        &mut itoa_buffer,
                        &ref_idx,
                        label.as_bytes(),
                    );
                }
                // (2) Reverse complementary Query
                {
                    let rev_com_query = reverse_complement_of_dna(&query);
                    let result = aligner.alignment(&reference, &mut sequence_buffer, &rev_com_query);
                    write_alignment_result_as_tsv::<ReverseDirection>(
                        result,
                        &mut buf_writer,
                        &mut itoa_buffer,
                        &ref_idx,
                        label.as_bytes(),
                    );
                }
            });
            eprintln!("   - Alignment {} s", start.elapsed().as_secs_f64());
        });
    }
}

trait Direction {
    const TAG : u8;
}
struct ForwardDirection;
impl Direction for ForwardDirection { const TAG : u8 = b'F'; }
struct ReverseDirection;
impl Direction for ReverseDirection { const TAG : u8 = b'R'; }

#[inline(always)]
fn write_alignment_result_as_tsv<D: Direction>(
    result: AlignmentResult,
    buf_writer: &mut BufWriter<StdoutLock>,
    itoa_buffer: &mut itoa::Buffer,
    ref_idx: &usize,
    label: &[u8],
) {
    result.0.into_iter().for_each(|TargetAlignmentResult {
        index: target_index,
        alignments: anchor_results,
    }| {
        anchor_results.into_iter().for_each(|anchor_result| {
            let _ = buf_writer.write(label).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(D::TAG).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(*ref_idx).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(target_index).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.penalty).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.length).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.position.query.0).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.position.query.1).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.position.target.0).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.position.target.1).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            anchor_result.operations.iter().for_each(|AlignmentOperations {
                operation,
                count,
            }| {
                let _ = buf_writer.write(
                    match operation {
                        AlignmentOperation::Match => b"M",
                        AlignmentOperation::Subst => b"S",
                        AlignmentOperation::Insertion => b"I",
                        AlignmentOperation::Deletion => b"D",
                    }
                ).unwrap();
                let _ = buf_writer.write(itoa_buffer.format(*count).as_bytes()).unwrap();
            });
            let _ = buf_writer.write(b"\n").unwrap();
        });
    });
}
