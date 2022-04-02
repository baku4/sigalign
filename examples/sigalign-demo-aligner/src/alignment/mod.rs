use std::{
    path::{PathBuf, Path},
    time::Instant, io::Write,   
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

#[derive(Debug)]
pub struct AlignmentConfig {
    // Path
    input_fasta_pathbuf: PathBuf,
    input_reference_pathbuf: PathBuf,
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
        let mut stdout = std::io::stdout();
        stdout.write(b"[").unwrap(); // Opening

        for (ref_idx, ref_path) in reference_paths.0.into_iter().enumerate() {
            eprintln!("  Reference {}", ref_idx);

            // Load reference
            let ref_load_start = Instant::now();
            let self_desc_reference = SelfDescReference::load_from_file(&ref_path).unwrap();
            eprintln!("   - Load reference {} s", ref_load_start.elapsed().as_secs_f64());

            // Alignment
            let do_align_start = Instant::now();
            if ref_idx != 0 {
                stdout.write(b",").unwrap();
            }
            match self_desc_reference {
                SelfDescReference::InMemory(inner_ref) => {
                    aligner.fasta_file_alignment_json_to_stream(
                        &inner_ref,
                        &config.input_fasta_pathbuf,
                        &mut stdout,
                    ).unwrap();
                },
                SelfDescReference::InMemoryRc(inner_ref) => {
                    aligner.fasta_file_alignment_json_to_stream(
                        &inner_ref,
                        &config.input_fasta_pathbuf,
                        &mut stdout,
                    ).unwrap();
                },
            }
            eprintln!("   - Alignment {} s", do_align_start.elapsed().as_secs_f64());
        }
        stdout.write(b"]").unwrap(); // Closing
        stdout.flush().unwrap();


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
