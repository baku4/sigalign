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

use super::SelfDescReference;
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
    use_rc: bool,
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
            .arg(arg!(-r --reference <FILE> "Define reference fasta(.fa, .fasta, .fna) or Sigalign reference file")
                .required(true))
            .arg(arg!(- -reverse  "Use reverse complementary sequence"))
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
        eprintln!(" - Time elapsed: {} s", start.elapsed().as_secs_f64());

        let start = Instant::now();
        eprintln!("# 2. Get reference");
        let self_desc_reference = config.get_reference().unwrap();
        eprintln!(" - Time elapsed: {} s", start.elapsed().as_secs_f64());

        let start = Instant::now();
        eprintln!("# 3. Make aligner");
        let mut aligner = config.make_aligner().unwrap();
        eprintln!(" - Time elapsed: {} s", start.elapsed().as_secs_f64());

        let start = Instant::now();
        eprintln!("# 4. Alignment");
        self_desc_reference.alignment(&mut aligner, &config.input_fasta_pathbuf).unwrap();
        eprintln!(" - Time elapsed: {} s", start.elapsed().as_secs_f64());

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

        let use_rc = matches.is_present("reverse");

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
                use_rc,
                px,
                po,
                pe,
                min_len,
                max_ppl,
                use_local_alg,
            }
        )
    }
    fn get_reference(&self) -> Result<SelfDescReference> {
        let use_new_ref = if let Some(ext) = self.input_reference_pathbuf.extension() {
            if let Some(ext) = ext.to_str() {
                vec!["fa", "fasta", "fna"].contains(&ext)
            } else {
                false
            }
        } else {
            false
        };
        
        if use_new_ref {
            eprintln!(" Make new reference from file {:?}", self.input_reference_pathbuf);
            if self.use_rc {
                let mut sequence_provider = InMemoryProvider::new();
                sequence_provider.add_fasta_file(&self.input_reference_pathbuf)?;
                Ok(SelfDescReference::InMemory(ReferenceBuilder::new().build(sequence_provider)?))
            } else {
                let mut sequence_provider = InMemoryRcProvider::new();
                sequence_provider.add_fasta_file(&self.input_reference_pathbuf)?;
                Ok(SelfDescReference::InMemoryRc(ReferenceBuilder::new().build(sequence_provider)?))
            }
        } else {
            eprintln!(" Load reference from file {:?}", self.input_reference_pathbuf);
            SelfDescReference::load_from_file(&self.input_reference_pathbuf)
        }
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

impl SelfDescReference {
    fn alignment(&self, aligner: &mut Aligner, fasta_file: &PathBuf) -> Result<()> {
        let result = match self {
            Self::InMemory(inner_ref) => {
                aligner.fasta_file_alignment(inner_ref, fasta_file)?.to_json()
            },
            Self::InMemoryRc(inner_ref) => {
                aligner.fasta_file_alignment(inner_ref, fasta_file)?.to_json()
            },
            Self::IndexedFasta(inner_ref) => {
                aligner.fasta_file_alignment(inner_ref, fasta_file)?.to_json()
            },
            Self::IndexedFastaRc(inner_ref) => {
                aligner.fasta_file_alignment(inner_ref, fasta_file)?.to_json()
            },
        };
        
        let mut stdout = std::io::stdout();
        stdout.write(result.as_bytes())?;
        stdout.flush()?;

        Ok(())
    }
}
