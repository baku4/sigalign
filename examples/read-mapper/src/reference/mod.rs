use crate::{error_msg, Result};
use clap::{arg, value_parser, Arg, ArgMatches, Command};
use sigalign::Reference;
use sigalign_core::reference::{Reference as RawReference, SequenceStorage};
use sigalign_impl::{
    pattern_index::dynamic_lfi::DynamicLfiOption, sequence_storage::in_memory::InMemoryStorage,
};
use sigalign_utils::{
    file_extension_checker::{is_fasta_file, is_gzip_file},
    sequence_reader::{decompress::get_gzip_decoder, fasta::FastaReader, IdRecord, SeqRecord},
};
use std::{
    fs::File,
    io::{BufWriter, Read},
    path::PathBuf,
    time::Instant,
};

mod path_detector;
pub use path_detector::ReferencePathDetector;
mod manifest;
pub use manifest::{write_manifest_header, write_manifest_line};

const DEFAULT_MAX_LENGTH: u32 = u32::MAX - 1;

pub struct ReferenceApp;

#[derive(Debug)]
struct Config {
    input_files: Vec<PathBuf>,
    output_file: PathBuf,
    // Maximum length of sequence for each reference file
    max_length: u32,
}

impl ReferenceApp {
    pub fn get_command() -> Command {
        Command::new("reference")
            .about("Generate reference file")
            .arg_required_else_help(true)
            .arg(
                arg!(-i --input <FILE> "Input file(s) path (FASTA format; can be gzipped)")
                    .display_order(1)
                    .num_args(1..)
                    .value_parser(value_parser!(PathBuf))
                    .required(true),
            )
            .arg(
                arg!(-o --output <FILE> "Output reference file path")
                    .display_order(2)
                    .value_parser(value_parser!(PathBuf))
                    .required(true),
            )
            .arg(
                arg!(-w --overwrite  "Overwrite output reference file")
                    .display_order(3)
                    .required(false),
            )
            .arg(
                Arg::new("maxlen")
                    .short('m')
                    .long("maxlen")
                    .help("Maximum length of sequence for each reference chunk (default: 2^32 - 2)")
                    .display_order(4)
                    .value_name("INT")
                    .value_parser(value_parser!(u32))
                    .required(false),
            )
    }
    pub fn run(matches: &ArgMatches) -> Result<()> {
        let total_start = Instant::now();

        // Parse configuration
        let config = {
            let start = Instant::now();
            let config = Config::from_matches(matches)?;
            eprintln!("Configuration parsed ({} s)", start.elapsed().as_secs_f64());
            eprintln!("{:#?}", config);
            config
        };

        // Build and save reference
        {
            let start = Instant::now();
            config.build_and_save_references()?;
            eprintln!(
                "Reference built and saved ({} s)",
                start.elapsed().as_secs_f64()
            );
        };

        eprintln!(
            "Total elapsed time: {:.8} s",
            total_start.elapsed().as_secs_f64()
        );
        Ok(())
    }
}

impl Config {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        // Input file paths
        let input_files = {
            let args = matches
                .get_many::<PathBuf>("input")
                .expect("Invalid input file");
            let input_files = args.into_iter().map(|p| p.clone()).collect::<Vec<_>>();

            for input_file in &input_files {
                if input_file.is_dir() {
                    error_msg!("Input file ({}) must be a file", input_file.display())
                } else if !input_file.exists() {
                    error_msg!("Input file ({}) does not exist", input_file.display())
                } else if !input_file_extension_is_allowed(input_file) {
                    error_msg!(
                        "Input file ({}) extension is not allowed",
                        input_file.display()
                    )
                }
            }

            input_files
        };

        // Output file path
        let overwrite = matches.get_flag("overwrite");
        let output_file = {
            matches
                .get_one::<PathBuf>("output")
                .expect("Invalid output file")
                .clone()
        };
        if output_file.is_dir() {
            error_msg!("Output file must be a file")
        } else {
            let reference_path_detector = ReferencePathDetector::new(&output_file);
            let to_clean_up_paths = reference_path_detector.to_clean_up_paths()?;
            if overwrite {
                eprintln!("Removing existing reference file(s)");
                for existing_reference_path in to_clean_up_paths {
                    eprintln!("  - Removing : {}", existing_reference_path.display());
                    std::fs::remove_file(existing_reference_path)?;
                }
            } else {
                eprintln!("Checking existing reference file(s)");
                if to_clean_up_paths.len() > 0 {
                    error_msg!("Output file already exist")
                } else {
                    eprintln!("No existing reference file(s)");
                }
            }
        }

        // Maximum length of reference
        let max_length = match matches.get_one::<u32>("maxlen") {
            Some(v) => {
                if *v == 0 {
                    error_msg!("Maximum length must be greater than 0")
                } else if *v > DEFAULT_MAX_LENGTH {
                    error_msg!("Maximum length must be less than {}", DEFAULT_MAX_LENGTH)
                } else {
                    *v
                }
            }
            None => DEFAULT_MAX_LENGTH,
        };

        Ok(Self {
            input_files,
            output_file,
            max_length,
        })
    }
    fn build_and_save_references(&self) -> Result<()> {
        let reference_path_detector = ReferencePathDetector::new(&self.output_file);
        let mut reference_index = 0;

        // Manifest file
        let mut manifest_file = File::create(reference_path_detector.get_manifest_file_path())?;
        let mut manifest_writer = BufWriter::new(&mut manifest_file);
        write_manifest_header(&mut manifest_writer)?;

        // Save reference by chunk
        let mut sequence_buffer = Vec::new();
        let mut label_buffer = String::new();
        let mut sequence_storage = InMemoryStorage::new();

        // For each input file
        for (input_file_index, input_file) in self.input_files.iter().enumerate() {
            let input_file_name = input_file
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            let readable_file: Box<dyn Read> = {
                let file = File::open(input_file)?;
                if is_gzip_file(input_file) {
                    Box::new(get_gzip_decoder(file))
                } else {
                    Box::new(file)
                }
            };
            let mut fasta_reader = FastaReader::new(readable_file);
            let mut record_index = 0;

            let mut target_index = sequence_storage.num_targets();
            let mut total_length = sequence_storage.get_total_length();

            while let Some(mut record) = fasta_reader.next() {
                sequence_buffer.clear();
                record.extend_seq_buf(&mut sequence_buffer);
                label_buffer.clear();
                record.extend_id_string(&mut label_buffer)?;

                write_manifest_line(
                    &mut manifest_writer,
                    &input_file_index,
                    input_file_name,
                    &record_index,
                    &reference_index,
                    &target_index,
                    label_buffer.as_str(),
                    &sequence_buffer.len(),
                )?;

                if (total_length != 0)
                    && (total_length + sequence_buffer.len() as u32 > self.max_length)
                {
                    // Build and Save
                    let reference = get_reference_with_default_option(sequence_storage)?;
                    let reference_path = reference_path_detector.get_path_of_index(reference_index);
                    eprintln!(
                        "Saving reference chunk index {} to {} ({} bp, {} targets)",
                        reference_index,
                        reference_path.display(),
                        reference.get_total_length(),
                        reference.get_num_targets(),
                    );
                    let out_file = File::create(reference_path)?;
                    reference.save_to(out_file)?;
                    reference_index += 1;
                    total_length = 0;

                    // Reset
                    sequence_storage = InMemoryStorage::new();
                }

                sequence_storage.add_target(&label_buffer, &sequence_buffer);

                total_length += sequence_buffer.len() as u32;
                record_index += 1;
                target_index += 1;
            }
        }

        // Save last reference
        if sequence_storage.num_targets() != 0 {
            let reference = get_reference_with_default_option(sequence_storage)?;
            let reference_path = reference_path_detector.get_path_of_index(reference_index);
            eprintln!(
                "Saving reference chunk index {} to {} ({} bp, {} targets)",
                reference_index,
                reference_path.display(),
                reference.get_total_length(),
                reference.get_num_targets(),
            );
            let out_file = File::create(reference_path)?;
            reference.save_to(out_file)?;
        }

        eprintln!(
            "Target manifest file has been saved to {}",
            reference_path_detector.get_manifest_file_path().display()
        );

        Ok(())
    }
}

fn get_reference_with_default_option(mut sequence_storage: InMemoryStorage) -> Result<Reference> {
    // Preparing sequence storage
    sequence_storage.set_sequences_to_uppercase();
    sequence_storage.change_bases_to(b"N", b'?');

    let total_length = sequence_storage.get_total_length();
    // Use half of total length as the maximum size of lookup table.
    // Maximum: 200 MiB
    let lookup_table_max_bytes_size = u64::min(200 * 1024 * 1024, (total_length / 8) as u64);
    let dynamic_lfi_option: DynamicLfiOption = DynamicLfiOption {
        suffix_array_sampling_ratio: 1,
        lookup_table_max_bytes_size,
        use_safe_guard: true,
    };
    let raw_reference = RawReference::new(sequence_storage, dynamic_lfi_option)?;
    let reference = Reference::from(raw_reference);
    Ok(reference)
}

fn input_file_extension_is_allowed(path: &PathBuf) -> bool {
    if is_gzip_file(path) {
        let path_without_gz = path.with_extension("");
        if is_fasta_file(path_without_gz) {
            return true;
        } else {
            return false;
        }
    } else if is_fasta_file(path) {
        return true;
    } else {
        return false;
    }
}
