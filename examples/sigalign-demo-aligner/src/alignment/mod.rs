use std::{
    path::{PathBuf, Path},
    time::Instant, io::Write,
    fs::File,
};

use super::{Result, format_err, error_msg};

use clap::{
    arg, App,
    ArgGroup,
    ArgMatches,
};

use super::{
    ReferencePaths,
    SelfDescReference,
};
use sigalign::{
    ReferenceBuilder,
    sequence_provider::{InMemoryProvider, InMemoryRcProvider},
    Aligner,
};

#[cfg(feature = "tsv")]
mod tsv;
#[cfg(feature = "tsv")]
use tsv::alignment_as_tsv_to_stdout;

#[derive(Debug)]
pub struct AlignmentConfig {
    // Path
    input_fasta_pathbuf: PathBuf,
    input_reference_pathbuf: PathBuf,
    output_json_pathbuf: PathBuf,
    // Condition
    px: usize,
    po: usize,
    pe: usize,
    min_len: usize,
    max_ppl: f32,
    // Algorithm
    use_local_alg: bool,
}

impl AlignmentConfig {
    pub fn add_args(app: App) -> App {
        app.about("Alignment with FASTA file")
            .arg(arg!(-s --semiglobal "Use semi-global algorithm"))
            .arg(arg!(-l --local "Use local algorithm"))
            .arg(arg!(-i --input <FILE> "Input query FASTA path"))
            .group(ArgGroup::new("algorithm")
                .required(true)
                .args(&["semiglobal", "local"]))
            .arg(arg!(-r --reference <FILE> "SigAlign reference file")
                .required(true))
            .arg(arg!(-o --output <FILE> "Output json path without extension. Output will be saved to {{output}}.{{ref_num}}.json")
                .required(true))
            .arg(arg!(-p --penalties "Mismatch, Gap-open and Gap-extend penalties")
                .value_names(&["MISM", "GOPN", "GEXT"])
                .required(true))
            .arg(arg!(-c --cutoff "Minimum aligned length and maximum penalty per length")
                .value_names(&["MINLEN", "MAXPPL"])
                .use_delimiter(true)
                .multiple_values(true)
                .required(true))
    }
    pub fn run_command(matches: &ArgMatches) {
        let total_start = Instant::now();

        let start = Instant::now();
        eprintln!("# 1. Parsing configuration");
        let config = Self::new_with_validation(matches).unwrap();
        let reference_paths = config.get_reference_paths();
        eprintln!(" - Time elapsed: {} s", start.elapsed().as_secs_f64());

        let start = Instant::now();
        eprintln!("# 2. Make aligner");
        let mut aligner = config.make_aligner().unwrap();
        eprintln!("{:#?}", aligner);
        eprintln!(" - Time elapsed: {} s", start.elapsed().as_secs_f64());

        eprintln!("# 3. Alignment");
        #[cfg(not(feature = "tsv"))]
        for (ref_idx, ref_path) in reference_paths.0.into_iter().enumerate() {
            eprintln!("  Reference {}", ref_idx);

            // Get output file path
            let mut output_json_pathbuf = config.output_json_pathbuf.clone();
            output_json_pathbuf.set_extension(format!("{}.json", ref_idx));
            eprintln!("  Output file path {:?}", output_json_pathbuf);
            let mut output_file = File::create(output_json_pathbuf).unwrap();

            let alignment_result = {
                // Load reference
                let ref_load_start = Instant::now();
                let self_desc_reference = SelfDescReference::load_from_file(&ref_path).unwrap();
                eprintln!("   - Load reference {} s", ref_load_start.elapsed().as_secs_f64());

                // Alignment
                let do_align_start = Instant::now();
                let alignment_result = match self_desc_reference {
                    SelfDescReference::Ref(inner_ref) => {
                        aligner.fasta_file_alignment_unchecked(
                            &inner_ref,
                            &config.input_fasta_pathbuf
                        ).unwrap()
                    },
                    SelfDescReference::RecRc(inner_ref) => {
                        aligner.fasta_file_alignment_unchecked(
                            &inner_ref,
                            &config.input_fasta_pathbuf
                        ).unwrap()
                    },
                };
                eprintln!("   - Alignment {} s", do_align_start.elapsed().as_secs_f64());
                alignment_result
            };

            // Serialize and save
            let serialize_start = Instant::now();
            let serialized = alignment_result.to_json();
            output_file.write_all(serialized.as_bytes()).unwrap();
            output_file.flush().unwrap();
            eprintln!("   - Saved to json file {} s", serialize_start.elapsed().as_secs_f64());
        }
        #[cfg(feature = "tsv")]
        {
            let mut stdout = std::io::stdout();
            eprintln!("  ! TSV feature is on. Ignore output params.");
            #[cfg(feature = "revcom")]
            eprintln!("  ! REVCOM feature is on.");

            for (ref_idx, ref_path) in reference_paths.0.into_iter().enumerate() {
                eprintln!("  Reference {}", ref_idx);

                // Load reference
                let ref_load_start = Instant::now();
                let self_desc_reference = SelfDescReference::load_from_file(&ref_path).unwrap();
                eprintln!("   - Load reference {} s", ref_load_start.elapsed().as_secs_f64());

                // Alignment
                let do_align_start = Instant::now();
                match self_desc_reference {
                    SelfDescReference::Ref(inner_ref) => {
                        alignment_as_tsv_to_stdout(
                            &mut aligner,
                            ref_idx,
                            inner_ref,
                            &config.input_fasta_pathbuf,
                            &mut stdout,
                        )
                    },
                    SelfDescReference::RecRc(inner_ref) => {
                        alignment_as_tsv_to_stdout(
                            &mut aligner,
                            ref_idx,
                            inner_ref,
                            &config.input_fasta_pathbuf,
                            &mut stdout,
                        )
                    },
                };
                stdout.flush().unwrap();
                eprintln!("   - Alignment {} s", do_align_start.elapsed().as_secs_f64());
            }
        }

        eprintln!("# 5. All processes are completed");
        eprintln!(" - Total time elapsed: {} s", total_start.elapsed().as_secs_f64());
    }
    fn new_with_validation(matches: &ArgMatches) -> Result<Self> {
        // (1) Path
        let input_fasta_path_str = matches.value_of("input")
            .ok_or(format_err!("Invalid input fasta"))?;
        let input_fasta_path = Path::new(input_fasta_path_str);
        let input_fasta_pathbuf = input_fasta_path.to_path_buf();

        let input_reference_path_str = matches.value_of("reference")
            .ok_or(format_err!("Invalid reference fasta"))?;
        let input_reference_path = Path::new(input_reference_path_str);
        let input_reference_pathbuf = input_reference_path.to_path_buf();

        let output_json_path_str = matches.value_of("output")
            .ok_or(format_err!("Invalid output path"))?;
        let output_json_path = Path::new(output_json_path_str);
        let output_json_pathbuf = output_json_path.to_path_buf();

        // (2) Condition
        let mut penalties = matches.values_of("penalties").unwrap();
        if penalties.len() != 3 {
            error_msg!("Penalties allow three positive integer")
        }
        let px: usize = penalties.next().unwrap().parse()?;
        let po: usize = penalties.next().unwrap().parse()?;
        let pe: usize = penalties.next().unwrap().parse()?;

        let mut cutoff = matches.values_of("cutoff").unwrap();
        if cutoff.len() != 2 {
            error_msg!("Cutoff receive positive integer and float")
        }
        let min_len: usize = cutoff.next().unwrap().parse()?;
        let max_ppl: f32 = cutoff.next().unwrap().parse()?;
        
        // (3) // Algorithm
        let use_local_alg = if matches.is_present("semiglobal") {
            false
        } else if matches.is_present("local") {
            true
        } else {
            error_msg!("Unknown algorithm")
        };

        Ok(
            Self {
                input_fasta_pathbuf,
                input_reference_pathbuf,
                output_json_pathbuf,
                px,
                po,
                pe,
                min_len,
                max_ppl,
                use_local_alg,
            }
        )
    }
    fn get_reference_paths(&self) -> ReferencePaths {
        let reference_paths = ReferencePaths::new_for_load(&self.input_reference_pathbuf);
        eprintln!(" Load reference from file");
        for path in &reference_paths.0 {
            eprintln!("{:?}", path);
        }

        reference_paths
    }
    fn make_aligner(&self) -> Result<Aligner> {
        if self.use_local_alg {
            Aligner::new_local(
                self.px,
                self.po,
                self.pe,
                self.min_len,
                self.max_ppl,
            )
        } else {
            Aligner::new_semi_global(
                self.px,
                self.po,
                self.pe,
                self.min_len,
                self.max_ppl,
            )
        }
    }
}
