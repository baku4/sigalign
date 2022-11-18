use super::{Result, error_msg};
use std::{
    path::PathBuf,
    time::Instant, fs::File,
    io::Write,
};
use clap::{
    builder::{Command, Arg},
    arg,
    ArgMatches,
    ArgGroup,
    value_parser,
};

use crate::{
    reference::{
        Reference,
        ReferencePaths,
    }
};
use sigalign::{
    Aligner as SigAligner,
    Reference as SigReference,
    core::ReferenceInterface,
    result::RecordAlignmentResult,
    util::FastaReader,
};

pub struct AlignmentApp;
#[derive(Debug, Clone)]
pub struct AlignmentConfig {
    // Path
    input_fasta_pathbuf: PathBuf,
    input_reference_paths: ReferencePaths,
    // Condition
    px: usize,
    po: usize,
    pe: usize,
    min_len: usize,
    max_ppl: f32,
    // Algorithm
    use_local_alg: bool,
}

impl AlignmentApp {
    pub fn get_command() -> Command {
        Command::new("alignment")
            .about("Alignment with FASTA file (print the result as TSV to stdout)")
            .arg_required_else_help(true)
            .arg(arg!(-s --semiglobal "Use semi-global algorithm").display_order(1))
            .arg(arg!(-l --local "Use local algorithm").display_order(2))
            .group(ArgGroup::new("algorithm")
                .required(true)
                .multiple(false)
                .args(["semiglobal", "local"]))
            .arg(arg!(-i --input <FILE> "Input query FASTA path").display_order(3)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-r --reference <FILE> "SigAlign reference file").display_order(4)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(Arg::new("penalties").short('p').long("penalties")
                .value_names(["INT", "INT", "INT"])
                .num_args(3)
                .help("Mismatch, Gap-open and Gap-extend penalties")
                .required(true)
                .display_order(6))
            .arg(Arg::new("cutoffs").short('c').long("cutoffs")
                .value_names(["INT", "FLOAT"])
                .num_args(2)
                .help("Minimum aligned length and maximum penalty per length")
                .required(true)
                .display_order(7))
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
            let px: usize = iterator.next().unwrap().parse().expect("Mismatch penalty allows integer.");
            let po: usize = iterator.next().unwrap().parse().expect("Gap-open penalty allows integer.");
            let pe: usize = iterator.next().unwrap().parse().expect("Gap-extend penalty allows integer.");

            (px, po, pe)
        };
        let (min_len, max_ppl) = {
            let mut iterator: clap::parser::ValuesRef<_> = matches.get_many::<String>("cutoffs").unwrap();
            let min_len: usize = iterator.next().unwrap().parse().expect("Cutoff of MINLEN allows integer.");
            let max_ppl: f32 = iterator.next().unwrap().parse().expect("Cutoff of MAXPPL allows float.");

            (min_len, max_ppl)
        };
        
        // (3) Algorithm
        let use_local_alg = if matches.contains_id("semiglobal") {
            false
        } else if matches.contains_id("local") {
            true
        } else {
            error_msg!("Unknown algorithm")
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
                use_local_alg,
            }
        )
    }
    fn make_aligner(&self) -> SigAligner {
        if self.use_local_alg {
            SigAligner::new_local(
                self.px,
                self.po,
                self.pe,
                self.min_len,
                self.max_ppl,
            )
        } else {
            SigAligner::new_semi_global(
                self.px,
                self.po,
                self.pe,
                self.min_len,
                self.max_ppl,
            )
        }.unwrap()
    }
    // TSV line format:
    // | query label | reference index | record index | penalty | length |
    //  query start position | query end position | record start position | record end position |
    //  string operations |
    fn perform_alignment(&self, mut aligner: SigAligner) {
        let mut stdout = std::io::stdout();

        self.input_reference_paths.0.iter().enumerate().for_each(|(ref_idx, ref_file_path)| {
            eprintln!("  Reference {}", ref_idx);
            let reference = {
                let start = Instant::now();
                let reference = Reference::load_from_file(ref_file_path).unwrap();
                eprintln!("   - Load reference {} s", start.elapsed().as_secs_f64());
                reference.inner
            };

            // Alignment
            let start = Instant::now();
            let mut sequence_buffer = reference.get_buffer();
            let fasta_reader = FastaReader::from_file_path(&self.input_fasta_pathbuf).unwrap();
            fasta_reader.for_each(|(label, query)| {
                let result = aligner.alignment(&reference, &mut sequence_buffer, &query);
    
                result.0.into_iter().for_each(|RecordAlignmentResult {
                    index: record_index,
                    alignments: anchor_results,
                }| {
                    anchor_results.into_iter().for_each(|anchor_result| {
                        let line = format!(
                            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                            label, ref_idx, record_index, anchor_result.penalty, anchor_result.length,
                            anchor_result.position.query.0, anchor_result.position.query.1,
                            anchor_result.position.record.0, anchor_result.position.record.1,
                            operations_to_string(&anchor_result.operations)
                        );
        
                        stdout.write(line.as_bytes()).unwrap();
                    });
                });
            });
            eprintln!("   - Alignment {} s", start.elapsed().as_secs_f64());
        });

        stdout.flush().unwrap();
    }
}

use sigalign::core::{
    AlignmentOperation,
    AlignmentCase,
};
fn operations_to_string(operations: &Vec<AlignmentOperation>) -> String {
    let string_ops: Vec<String> = operations.iter().map(|op| {
        format!(
            "{}{}",
            match op.case {
                AlignmentCase::Match => 'M',
                AlignmentCase::Subst => 'S',
                AlignmentCase::Insertion => 'I',
                AlignmentCase::Deletion => 'D',
            },
            op.count,
        )
    }).collect();
    string_ops.concat()
}