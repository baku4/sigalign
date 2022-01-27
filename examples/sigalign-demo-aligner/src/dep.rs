use sigalign::{
    Reference as SigReference,
    ReferenceBuilder as SigReferenceBuilder,
    Aligner as SigAligner,
};
use sigalign::sequence_provider::{
    InMemoryProvider,
    IndexedFastaProvider,
};
use sigalign::util::FastaReader;

use clap::{App, SubCommand, Arg, ArgMatches};
use anyhow::{Result, bail as error_msg};

use std::path::Path;
use std::fs::File;
use std::fs::metadata;
use std::time::{Duration, Instant};

pub struct Configuration;

impl Configuration {
    pub fn get_matches() -> ArgMatches<'static> {
        let app = App::new("SigAlign demo aligner")
            .version("0.1.0")
            .author("baku4 <bahkhun@gamil.com>")
            .about("Alignment query to target reference");
        
        let reference_subcommand = SubCommand::with_name("reference")
            .about("Generate reference file")
            .arg(Arg::with_name("input")
                    .long("input")
                    .short("i")
                    .help("Path of input Fasta file")
                    .value_name("PATH")
                    .required(true)
                    .takes_value(true)
                    .display_order(1))
            .arg(Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .help("Path of reference file to output")
                    .value_name("PATH")
                    .required(true)
                    .takes_value(true)
                    .display_order(2))
            .arg(Arg::with_name("not_in_memory")
                    .long("nomem")
                    .short("m")
                    .help("Use provider of index-fasta instead of in-memory")
                    .display_order(3))
            .arg(Arg::with_name("reverse_complement")
                    .long("reverse")
                    .short("r")
                    .help("Whether to add reverse complementary sequence")
                    .display_order(4))
            .arg(Arg::with_name("higher_compressed_bwt")
                    .long("compressed")
                    .short("c")
                    .help("Whether to use higher compressed (64 to 128) Bwt block")
                    .display_order(5))
            .arg(Arg::with_name("suffix_array_sampling_ratio")
                    .long("sampling")
                    .short("s")
                    .help("Sampling ratio for suffix array")
                    .value_name("INT")
                    .takes_value(true)
                    .display_order(6))
            .arg(Arg::with_name("kmer_size_for_lookup_table")
                    .long("kmer")
                    .short("k")
                    .help("Kmer size for count array lookup table")
                    .value_name("INT")
                    .takes_value(true)
                    .display_order(7));
        
        let alignment_args = [
            // Input file path
            Arg::with_name("reference")
                .long("reference")
                .short("r")
                .help("Reference fasta file path")
                .value_name("PATH")
                .required(false)
                .takes_value(true)
                .display_order(1),
            Arg::with_name("query")
                .long("query")
                .short("q")
                .help("Fasta file to query")
                .value_name("PATH")
                .required(true)
                .takes_value(true)
                .display_order(2),
            // Aligner option
            Arg::with_name("penalties")
                .long("penalties")
                .short("p")
                .help("Penalty for mismatch, gap-open and gap-extend")
                .value_name("[INT,INT,INT]")
                .multiple(true)
                .use_delimiter(true)
                .required(true)
                .takes_value(true)
                .display_order(3),
            Arg::with_name("min_aligned_length")
                .long("min_len")
                .short("ml")
                .help("Minimum aligned length")
                .value_name("INT")
                .required(true)
                .takes_value(true)
                .display_order(4),
            Arg::with_name("max_penalty_per_length")
                .long("max_ppl")
                .short("mp")
                .help("Maximum penalty per length (0~1)")
                .value_name("FLOAT")
                .required(true)
                .takes_value(true)
                .display_order(5),
            Arg::with_name("with_label")
                .long("with_label")
                .short("w")
                .help("Return labeled result")
                .display_order(6),
            // Reference option
            Arg::with_name("reverse_complement")
                .long("reverse")
                .short("rc")
                .help("Whether to add reverse complementary sequence")
                .display_order(7),
            Arg::with_name("higher_compressed_bwt")
                .long("compressed")
                .short("c")
                .help("Whether to use higher compressed (64 to 128) Bwt block")
                .display_order(8),
            Arg::with_name("suffix_array_sampling_ratio")
                .long("sampling")
                .short("s")
                .help("Sampling ratio for suffix array")
                .value_name("INT")
                .takes_value(true)
                .display_order(9),
            Arg::with_name("kmer_size_for_lookup_table")
                .long("kmer")
                .short("k")
                .help("Kmer size for count array lookup table")
                .value_name("INT")
                .takes_value(true)
                .display_order(10),
        ];

        let semiglobal_subcommand = SubCommand::with_name("semiglobal")
            .about("Semi-global alignment")
            .args(&alignment_args);
        let local_subcommand = SubCommand::with_name("local")
            .about("Local alignment")
            .args(&alignment_args);
        
        let matches = app.subcommand(reference_subcommand.display_order(1))
            .subcommand(semiglobal_subcommand.display_order(2))
            .subcommand(local_subcommand.display_order(3))
            .get_matches();
        
        matches
    }
    pub fn interpret(matches: &ArgMatches) {
        match matches.subcommand() {
            ("semiglobal",  Some(sub_matches)) => {
                eprintln!("# Semi-global alignment with Sigalign");
                Self::do_alignment(sub_matches, true).unwrap();
            },
            ("local",  Some(sub_matches)) => {
                eprintln!("# Local alignment with Sigalign");
                Self::do_alignment(sub_matches, false).unwrap();
            },
            _ => panic!("Not support subcommand")
        }
    }
    fn do_alignment(
        matches: &ArgMatches,
        is_semi_global: bool,
    ) -> Result<()>{
        // ####################################
        // 1. Parse parameters
        let total_start_time = Instant::now();
        eprintln!("#1. Parsing parameters");
        // (1) File path
        let query_path = matches.value_of("query")
            .expect("Input Fasta path is not valid");
        let reference_path = matches.value_of("reference")
            .expect("Reference Fasta path is not valid");
        // (2) For Aligner
        let penalties = {
            let mut values = matches.values_of("penalties")
                .expect("Penalty values are not valid");
            let mismatch = values.next()
                .expect("Mismatch penalty is not valid").parse::<usize>().unwrap();
            let gap_open = values.next()
                .expect("Gap-open penalty is not valid").parse::<usize>().unwrap();
            let gap_extend = values.next()
            .expect("Gap-extend penalty is not valid").parse::<usize>().unwrap();

            (mismatch, gap_open, gap_extend)
        };
        let min_aligned_length = matches.value_of("min_aligned_length")
            .expect("Minimum aligned length is not valid").parse::<usize>()?;
        let max_penalty_per_length = matches.value_of("max_penalty_per_length")
            .expect("Maximum penalty per length is not valid").parse::<f32>()?;
        let with_label = matches.is_present("with_label");
        // (2) For Aligner
        let use_reverse_complement = matches.is_present("reverse_complement");
        let use_128_bwt = matches.is_present("higher_compressed_bwt");
        let sa_sampling_ratio = match matches.value_of("suffix_array_sampling_ratio") {
            Some(value) => Some(
                value.parse::<u64>().expect("Specified sampling ratio must be integer")
            ),
            None => None,
        };
        let kmer_size = match matches.value_of("kmer_size_for_lookup_table") {
            Some(value) => Some(
                value.parse::<usize>().expect("Kmer size for lookup table must be integer")
            ),
            None => None,
        };
        let preparing_data_time_elapsed = total_start_time.elapsed().as_secs_f32();
        eprintln!("Time elapsed to prepare parameters: {:?} s", preparing_data_time_elapsed);

        // ####################################
        // 2. Make reference
        let start_time = Instant::now();
        eprintln!("#2. Make reference");

        let lt_fm_index_config = {
            let mut config = LtFmIndexConfig::new();
            if use_128_bwt {
                config = config.use_bwt_size_of_128();
            }
            if let Some(value) = sa_sampling_ratio {
                config = config.change_sampling_ratio(value);
            }
            if let Some(value) = kmer_size {
                config = config.change_kmer_size_for_lookup_table(value);
            }
            config
        };
        let sequence_provider = if use_reverse_complement {
            InMemoryProvider::from_fasta_file_of_nucleotide_with_reverse_complement(reference_path)
        } else {
            InMemoryProvider::from_fasta_file(reference_path)
        }?;
        let mut reference = Reference::new_with_lt_fm_index_config(
            lt_fm_index_config,
            sequence_provider
        )?;
        eprintln!("Time elapsed to make reference: {} s", start_time.elapsed().as_secs_f32());

        // ####################################
        // 3. Make Aligner
        let start_time = Instant::now();
        eprintln!("#3. Make Aligner");

        let mut aligner = Aligner::new(
            penalties.0,
            penalties.1,
            penalties.2,
            min_aligned_length,
            max_penalty_per_length
        )?;
        eprintln!("Aligner: {:#?}", aligner);
        eprintln!("Time elapsed to make aligner: {:?} s", start_time.elapsed().as_secs_f32());

        // ####################################
        // 4. Alignment
        let start_time = Instant::now();
        eprintln!("#4. Alignment");
        if is_semi_global {
            if with_label {
                Self::semi_global_alignment_labeled(&mut aligner, &mut reference, query_path)?;
            } else {
                Self::semi_global_alignment(&mut aligner, &mut reference, query_path)?;
            }
        } else {
            if with_label {
                Self::local_alignment_labeled(&mut aligner, &mut reference, query_path)?;
            } else {
                Self::local_alignment(&mut aligner, &mut reference, query_path)?;
            }
        };

        eprintln!("Time elapsed to alignment: {:?} s", start_time.elapsed().as_secs_f32());
        eprintln!("Total time elapsed: {:?} s", total_start_time.elapsed().as_secs_f32());
        Ok(())
    }
    fn semi_global_alignment_labeled(
        aligner: &mut Aligner,
        reference: &mut Reference,
        query_path: &str,
    ) -> Result<()> {
        let result = aligner.semi_global_alignment_labeled_from_fasta_file(reference, query_path)?;
        print!("{}", result);
        Ok(())
    }
    fn semi_global_alignment(
        aligner: &mut Aligner,
        reference: &mut Reference,
        query_path: &str,
    ) -> Result<()> {
        let result = aligner.semi_global_alignment_from_fasta_file(reference, query_path)?;
        print!("{}", result);
        Ok(())
    }
    fn local_alignment_labeled(
        aligner: &mut Aligner,
        reference: &mut Reference,
        query_path: &str,
    ) -> Result<()> {
        let result = aligner.local_alignment_labeled_from_fasta_file(reference, query_path)?;
        print!("{}", result);
        Ok(())
    }
    fn local_alignment(
        aligner: &mut Aligner,
        reference: &mut Reference,
        query_path: &str,
    ) -> Result<()> {
        let result = aligner.local_alignment_from_fasta_file(reference, query_path)?;
        print!("{}", result);
        Ok(())
    }
}

type Reference = SigReference<InMemoryProvider>;