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

mod two_step_align;
use two_step_align::two_step_alignment_to_stdout;

#[derive(Debug)]
pub struct TSAlignmentConfig {
    // Path
    input_fasta_pathbuf: PathBuf,
    input_reference_pathbuf: PathBuf,
    // Condition
    px_1: usize,
    po_1: usize,
    pe_1: usize,
    min_len_1: usize,
    max_ppl_1: f32,
    px_2: usize,
    po_2: usize,
    pe_2: usize,
    min_len_2: usize,
    max_ppl_2: f32,
    // Algorithm
    use_local_alg: bool,
}

impl TSAlignmentConfig {
    pub fn add_args(app: App) -> App {
        app.about("Two-steps Alignment with FASTA file")
            .arg(arg!(-s --semiglobal "Use semi-global algorithm"))
            .arg(arg!(-l --local "Use local algorithm"))
            .arg(arg!(-i --input <FILE> "Input query FASTA path"))
            .group(ArgGroup::new("algorithm")
                .required(true)
                .args(&["semiglobal", "local"]))
            .arg(arg!(-r --reference <FILE> "SigAlign reference file")
                .required(true))
            .arg(arg!(-p1 --penalties_1 "Mismatch, Gap-open and Gap-extend penalties (1st)")
                .value_names(&["MISM", "GOPN", "GEXT"])
                .required(true))
            .arg(arg!(-c1 --cutoff_1 "Minimum aligned length and maximum penalty per length (1st)")
                .value_names(&["MINLEN", "MAXPPL"])
                .use_delimiter(true)
                .multiple_values(true)
                .required(true))
            .arg(arg!(-p2 --penalties_2 "Mismatch, Gap-open and Gap-extend penalties (2nd)")
                .value_names(&["MISM", "GOPN", "GEXT"])
                .required(true))
            .arg(arg!(-c2 --cutoff_2 "Minimum aligned length and maximum penalty per length (2nd)")
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
        let (mut aligner_1, mut aligner_2) = config.make_aligners().unwrap();
        eprintln!("1st Aligner:\n{:#?}", aligner_1);
        eprintln!("2st Aligner:\n{:#?}", aligner_2);
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
            two_step_alignment_to_stdout(
                self_desc_reference,
                &mut aligner_1,
                &mut aligner_2,
                &config.input_fasta_pathbuf,
                &mut stdout,
            ).unwrap();
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
        let mut penalties_1 = matches.values_of("penalties_1").unwrap();
        let mut penalties_2 = matches.values_of("penalties_2").unwrap();
        if penalties_1.len() != 3 {
            error_msg!("Penalties allow three positive integer")
        }
        if penalties_2.len() != 3 {
            error_msg!("Penalties allow three positive integer")
        }
        let px_1: usize = penalties_1.next().unwrap().parse()?;
        let po_1: usize = penalties_1.next().unwrap().parse()?;
        let pe_1: usize = penalties_1.next().unwrap().parse()?;
        let px_2: usize = penalties_2.next().unwrap().parse()?;
        let po_2: usize = penalties_2.next().unwrap().parse()?;
        let pe_2: usize = penalties_2.next().unwrap().parse()?;

        let mut cutoff_1 = matches.values_of("cutoff_1").unwrap();
        let mut cutoff_2 = matches.values_of("cutoff_2").unwrap();
        if cutoff_1.len() != 2 {
            error_msg!("Cutoff receive positive integer and float")
        }
        if cutoff_2.len() != 2 {
            error_msg!("Cutoff receive positive integer and float")
        }
        let min_len_1: usize = cutoff_1.next().unwrap().parse()?;
        let max_ppl_1: f32 = cutoff_1.next().unwrap().parse()?;
        let min_len_2: usize = cutoff_2.next().unwrap().parse()?;
        let max_ppl_2: f32 = cutoff_2.next().unwrap().parse()?;
        
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
                px_1,
                po_1,
                pe_1,
                min_len_1,
                max_ppl_1,
                px_2,
                po_2,
                pe_2,
                min_len_2,
                max_ppl_2,
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
    fn make_aligners(&self) -> Result<(Aligner, Aligner)> {
        if self.use_local_alg {
            Ok((
                Aligner::new_local(
                    self.px_1,
                    self.po_1,
                    self.pe_1,
                    self.min_len_1,
                    self.max_ppl_1,
                )?,
                Aligner::new_local(
                    self.px_2,
                    self.po_2,
                    self.pe_2,
                    self.min_len_2,
                    self.max_ppl_2,
                )?,
            ))
        } else {
            Ok((
                Aligner::new_semi_global(
                    self.px_1,
                    self.po_1,
                    self.pe_1,
                    self.min_len_1,
                    self.max_ppl_1,
                )?,
                Aligner::new_semi_global(
                    self.px_2,
                    self.po_2,
                    self.pe_2,
                    self.min_len_2,
                    self.max_ppl_2,
                )?,
            ))
        }
    }
}
