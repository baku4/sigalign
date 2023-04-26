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
        SigReferenceWrapper,
        ReferencePaths,
    }
};
use sigalign::{
    aligner::{
        AlignerInterface,
        LocalAligner, LinearStrategy,
    },
    results::{
        fasta::{
            FastaAlignmentResult,
            FastaReverseComplementAlignmentResult,
            ReadAlignmentResult,
            ReadReverseComplementAlignmentResult,
        },
    }, reference::ReferenceInterface,
    utils::{
        FastaReader,
        reverse_complement_of_dna,
    },
};
type SigAligner = LocalAligner<LinearStrategy>;

pub struct AlignmentApp;
#[derive(Debug, Clone)]
pub struct AlignmentConfig {
    // Path
    input_fasta_pathbuf: PathBuf,
    input_reference_paths: ReferencePaths,
    output_json_pathbuf: PathBuf,
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
            .about("Alignment with FASTA file")
            .arg_required_else_help(true)
            .arg(arg!(-i --input <FILE> "Input query FASTA path").display_order(1)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-r --reference <FILE> "SigAlign reference file").display_order(2)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-o --output <FILE> "Output json path without extension. Output will be saved to {output}.{ref_num}.json").display_order(3)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(Arg::new("penalties").short('p').long("penalties")
                .value_names(["INT", "INT", "INT"])
                .num_args(3)
                .help("Mismatch, Gap-open and Gap-extend penalties")
                .required(true)
                .display_order(5))
            .arg(Arg::new("cutoffs").short('c').long("cutoffs")
                .value_names(["INT", "FLOAT"])
                .num_args(2)
                .help("Minimum aligned length and maximum penalty per length")
                .required(true)
                .display_order(5))
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
        let output_json_pathbuf = {
            matches.get_one::<PathBuf>("output").expect("Invalid output path").clone()
        };

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
                output_json_pathbuf,
                px,
                po,
                pe,
                min_len,
                max_ppl,
            }
        )
    }
    fn make_aligner(&self) -> SigAligner {
        SigAligner::new(
            self.px,
            self.po,
            self.pe,
            self.min_len,
            self.max_ppl,
        ).unwrap()
    }
    fn perform_alignment(&self, mut aligner: SigAligner) {
        self.input_reference_paths.0.iter().enumerate().for_each(|(ref_idx, ref_file_path)| {
            eprintln!("  Reference {}", ref_idx);
            let reference = {
                let start = Instant::now();
                let reference = SigReferenceWrapper::load_from_file(ref_file_path).unwrap();
                eprintln!("   - Load reference {} s", start.elapsed().as_secs_f64());
                reference
            };

            // Get output file path
            let mut output_json_pathbuf = self.output_json_pathbuf.clone();
            output_json_pathbuf.set_extension(format!("{}.json", ref_idx));
            eprintln!("   - Output file path {:?}", output_json_pathbuf);
            let mut output_file = File::create(output_json_pathbuf).unwrap();

            // Get result
            let result = {
                let start = Instant::now();
                let result = align_fasta_file_with_rc_dna(&mut aligner, &reference, &self.input_fasta_pathbuf);
                eprintln!("   - Alignment {} s", start.elapsed().as_secs_f64());
                result
            };

            // Serialize and save
            {
                let start = Instant::now();
                output_file.write_all(result.to_json().as_bytes()).unwrap();
                output_file.flush().unwrap();
                eprintln!("   - Saved to json file {} s", start.elapsed().as_secs_f64());
            }
        });
    }
}

#[inline]
fn align_fasta_file_with_rc_dna(
    aligner: &mut SigAligner,
    reference: &SigReferenceWrapper,
    input_fasta: &PathBuf,
) -> FastaReverseComplementAlignmentResult {
    let mut sequence_buffer = reference.as_ref().get_buffer();
    let fasta_reader = FastaReader::from_path(input_fasta).unwrap();
    let mut results = Vec::new();
    fasta_reader.for_each(|(label, query)| {
        let result = aligner.alignment(
            &reference.inner,
            &mut sequence_buffer,
            &query,
        );
        results.push(ReadReverseComplementAlignmentResult {
            read: label.clone(),
            is_forward: true,
            result,
        });

        let rev_query = reverse_complement_of_dna(&query);
        let result = aligner.alignment(
            &reference.inner,
            &mut sequence_buffer,
            &rev_query,
        );
        results.push(ReadReverseComplementAlignmentResult {
            read: label.clone(),
            is_forward: false,
            result,
        });
    });
    
    FastaReverseComplementAlignmentResult(results)
}
