use super::{Result, format_err, error_msg};

use std::{
    path::{Path, PathBuf},
    time::Instant,
};


use clap::{
    arg, App,
    ArgGroup,
    ArgMatches,
};

use sigalign::ReferenceBuilder;
use sigalign::sequence_provider::{
    InMemoryProvider, InMemoryRcProvider, IndexedFastaProvider, IndexedFastaRcProvider,
};

#[derive(Debug, Clone)]
pub struct ReferenceConfig {
    // Path
    input_file_pathbuf: PathBuf,
    output_file_pathbuf: PathBuf,
    overwrite: bool,
    // Sequence provider type
    use_indexed_fasta: bool,
    use_rc: bool,
    // Pattern finder config
    use_128_bwt: bool,
    kmer: Option<usize>,
    sa_sampling_ratio: Option<u64>,
    // Sequence type config
    for_aminoacid: Option<bool>,
    noise: Option<u8>,
}

mod self_desc_ref;
pub use self_desc_ref::SelfDescReference;

impl ReferenceConfig {
    pub fn add_args(app: App) -> App {
        app.about("Generate reference file")
            .arg(arg!(-i --input <FILE> "Input FASTA path"))
            .arg(arg!(-o --output <FILE> "Output reference path"))
            .arg(arg!(-w - -overwrite  "Overwrite output reference file"))
            .arg(arg!(-r - -reverse  "Use reverse complementary sequence"))
            .arg(arg!(-c - -cpb  "Use higher compressed (128) Bwt block"))
            .arg(arg!(-s --ssr <INT>  "Suffix array sampling ratio")
                .required(false))
            .arg(arg!(-k --klt <INT> "Kmer size for count array lookup table")
                .required(false))
            .arg(arg!(-m - -nom  "Use index-fasta provider instead of in-memory")
                .required(false))
            .arg(arg!(--no "Define sequence type as nucleotide only")
                .required(false))
            .arg(arg!(--ao "Define sequence type as amino-acid only")
                .required(false))
            .arg(arg!(--nn <CHAR> "Define sequence type as nucleotide with noise")
                .required(false))
            .arg(arg!(--an <CHAR> "Define sequence type as amino-acid with noise")
                .required(false))
            .group(ArgGroup::new("text_type")
                .args(&["no", "ao", "nn", "an"])
                .required(false))
    }
    pub fn run_command(matches: &ArgMatches) {
        let total_start = Instant::now();

        let start = Instant::now();
        eprintln!("# 1. Parsing configuration");
        let config = Self::new_with_validation(matches).unwrap();
        eprintln!(" - Time elapsed: {} s", start.elapsed().as_secs_f64());

        let start = Instant::now();
        eprintln!("# 2. Make reference");
        let self_desc_reference = config.make_reference().unwrap();
        eprintln!(" - Time elapsed: {} s", start.elapsed().as_secs_f64());

        let start = Instant::now();
        eprintln!("# 3. Save reference");
        self_desc_reference.save_to_file(&config.output_file_pathbuf).unwrap();
        eprintln!(" - Time elapsed: {} s", start.elapsed().as_secs_f64());

        eprintln!("# 4. All processes are completed");
        eprintln!(" - Total time elapsed: {} s", total_start.elapsed().as_secs_f64());
    }
    fn new_with_validation(matches: &ArgMatches) -> Result<Self> {
        // (1) Path
        let input_file_path_str = matches.value_of("input")
            .ok_or(format_err!("Invalid input file"))?;
        let input_file_path = Path::new(input_file_path_str);
        let input_file_pathbuf = input_file_path.to_path_buf();

        let overwrite = matches.is_present("overwrite");

        let output_file_path_str = matches.value_of("output")
            .ok_or(format_err!("Invalid output file"))?;
        let output_file_path = Path::new(output_file_path_str);
        let output_file_pathbuf = output_file_path.to_path_buf();

        if !overwrite && output_file_path.exists() {
            error_msg!("Output file already exist")
        }

        // (2) Sequence provider type
        let in_memory = matches.is_present("nom");
        let use_rc = matches.is_present("reverse");
        
        // (3) Pattern finder config
        let use_128_bwt = matches.is_present("cpb");
        let kmer = match matches.value_of("klt") {
            Some(v) => Some(v.parse::<usize>()?),
            None => None,
        };
        let sa_sampling_ratio = match matches.value_of("ssr") {
            Some(v) => Some(v.parse::<u64>()?),
            None => None,
        };
        
        // Sequence type config
        let for_aminoacid = if matches.is_present("no") || matches.is_present("nn") {
            Some(false)
        } else if matches.is_present("ao") || matches.is_present("an") {
            Some(true)
        } else {
            None
        };
        let noise = if matches.is_present("nn") {
            let v = matches.value_of("nn").unwrap();
            Some(v.as_bytes()[0])
        } else if matches.is_present("an") {
            let v = matches.value_of("an").unwrap();
            Some(v.as_bytes()[0])
        } else {
            None
        };

        Ok(
            Self {
                input_file_pathbuf,
                output_file_pathbuf,
                overwrite,
                use_indexed_fasta: in_memory,
                use_rc,
                use_128_bwt,
                kmer,
                sa_sampling_ratio,
                for_aminoacid,
                noise,
            }
        )
    }
    fn make_reference(&self) -> Result<SelfDescReference> {
        let mut reference_builder = ReferenceBuilder::new();

        if self.use_128_bwt {
            reference_builder = reference_builder.change_bwt_vector_size_to_128();
        } else {
            reference_builder = reference_builder.change_bwt_vector_size_to_64();
        };

        if let Some(kmer) = self.kmer {
            reference_builder = reference_builder.change_count_array_kmer(kmer)?;
        };

        if let Some(sa_sampling_ratio) = self.sa_sampling_ratio {
            reference_builder = reference_builder.change_suffix_array_sampling_ratio(sa_sampling_ratio)?;
        };

        if let Some(for_aminoacid) = self.for_aminoacid {
            if for_aminoacid {
                if let Some(noise) = self.noise {
                    reference_builder = reference_builder.search_for_amino_acid_with_noise(noise);
                } else {
                    reference_builder = reference_builder.search_for_amino_acid_only();
                }
            } else {
                if let Some(noise) = self.noise {
                    reference_builder = reference_builder.search_for_nucleotide_with_noise(noise);
                } else {
                    reference_builder = reference_builder.search_for_nucleotide_only();
                }
            }
        };

        if self.use_indexed_fasta {
            if self.use_rc {
                let sp = IndexedFastaRcProvider::new(self.input_file_pathbuf.clone())?;
                Ok(SelfDescReference::IndexedFastaRc(reference_builder.build(sp)?))
            } else {
                let sp = IndexedFastaProvider::new(self.input_file_pathbuf.clone())?;
                Ok(SelfDescReference::IndexedFasta(reference_builder.build(sp)?))
            }
        } else {
            if self.use_rc {
                let mut sp = InMemoryRcProvider::new();
                sp.add_fasta_file(self.input_file_pathbuf.clone())?;
                Ok(SelfDescReference::InMemoryRc(reference_builder.build(sp)?))
            } else {
                let mut sp = InMemoryProvider::new();
                sp.add_fasta_file(self.input_file_pathbuf.clone())?;
                Ok(SelfDescReference::InMemory(reference_builder.build(sp)?))
            }
        }
    }
}
