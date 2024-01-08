use std::{
    path::PathBuf,
    time::Instant, fs::File,
    io::{Read, Write as _, BufWriter},
};

use clap::{
    Command,
    arg,
    Arg,
    ArgMatches,
    value_parser,
};

use crate::{Result, error_msg};
use super::ReferencePathDetector;
use sigalign::Reference;
use sigalign_core::reference::{
    SequenceStorage,
    Reference as RawReference,
};
use sigalign_utils::{
    file_extension_checker::{is_fasta_file, is_gzip_file},
    sequence_reader::decompress::get_gzip_decoder,
};
use sigalign_impl::{
    pattern_index::dynamic_lfi::DynamicLfiOption,
    sequence_storage::in_memory::InMemoryStorage,
};

const MAXIMUM_LENGTH: u32 = u32::MAX - 1;

pub struct ReferenceApp;
#[derive(Debug, Clone)]
struct ReferenceConfig {
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
            .arg(arg!(-i --input <FILE> "Input fasta files")
                .display_order(1)
                .num_args(1..)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-o --output <FILE> "Output reference path")
                .display_order(2)
                .value_parser(value_parser!(PathBuf))
                .required(true))
            .arg(arg!(-w --overwrite  "Overwrite output reference file")
                .display_order(3)
                .required(false))
            .arg(Arg::new("maxlen").short('m').long("maxlen")
                .help("Maximum length of sequence for each reference file")
                .display_order(4)
                .value_name("INT")
                .value_parser(value_parser!(u32))
                .required(false))
    }
    pub fn run(matches: &ArgMatches) -> Result<()> {
        let total_start = Instant::now();

        // Parse configuration
        let config = {
            let start = Instant::now();
            let config = ReferenceConfig::from_matches(matches)?;
            eprintln!("Configuration parsed ({} s)", start.elapsed().as_secs_f64());
            eprintln!("{:#?}", config);
            config
        };

        // Build and save reference
        {
            let start = Instant::now();
            config.build_and_save_references()?;
            eprintln!("Reference built and saved ({} s)", start.elapsed().as_secs_f64());
        };
        
        eprintln!("Total elapsed time: {} s", total_start.elapsed().as_secs_f64());
        Ok(())
    }
}

impl ReferenceConfig {
    fn from_matches(matches: &ArgMatches) -> Result<Self> {
        // Input file paths
        let input_files = {
            let args = matches.get_many::<PathBuf>("input").expect("Invalid input file");
            let input_files = args.into_iter().map(|p| p.clone()).collect::<Vec<_>>();

            for input_file in &input_files {
                if input_file.is_dir() {
                    error_msg!("Input file ({}) must be a file", input_file.display())
                } else if !input_file.exists() {
                    error_msg!("Input file ({}) does not exist", input_file.display())
                } else if !input_file_extension_is_allowed(input_file) {
                    error_msg!("Input file ({}) extension is not allowed", input_file.display())
                }
            };
            
            input_files
        };

        // Output file path
        let overwrite = matches.get_flag("overwrite");
        let output_file = {
            matches.get_one::<PathBuf>("output").expect("Invalid output file").clone()
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
                } else if *v > MAXIMUM_LENGTH {
                    error_msg!("Maximum length must be less than {}", MAXIMUM_LENGTH)
                } else {
                    *v
                }
            },
            None => {
                MAXIMUM_LENGTH
            },
        };

        Ok(
            Self {
                input_files,
                output_file,
                max_length,
            }
        )
    }
    fn build_and_save_references(&self) -> Result<()> {
        let reference_path_detector = ReferencePathDetector::new(&self.output_file);
        let mut current_index_of_reference = 0;

        let mut num_targets_in_input_files = Vec::new();
        let mut num_targets_in_references = Vec::new();

        let mut sequence_storage = InMemoryStorage::new();

        for input_file in &self.input_files {
            let before_fill_num_targets = sequence_storage.num_targets();
            let readable_file: Box<dyn Read> = {
                let file = File::open(input_file)?;
                if is_gzip_file(input_file) {
                    Box::new(get_gzip_decoder(file))
                } else {
                    Box::new(file)
                }
            };
            let filled_sequence_storages = sequence_storage.fill_fasta_until_max_length(
                readable_file,
                self.max_length,
            )?;
            let after_fill_num_targets = sequence_storage.num_targets();
            let num_targets_in_input_file = {
                let mut v = after_fill_num_targets - before_fill_num_targets;
                filled_sequence_storages.iter().for_each(|s| {
                    v += s.num_targets();
                });
                v
            };
            num_targets_in_input_files.push(num_targets_in_input_file);

            for filled_sequence_storage in filled_sequence_storages {
                let reference_path = reference_path_detector.get_path_of_index(current_index_of_reference);
                let reference = get_reference_with_default_option(filled_sequence_storage)?;
                reference.get_total_length();
                num_targets_in_references.push(reference.get_num_targets());

                eprintln!(
                    "Saving reference chunk index {} to {} ({} bp, {} targets)",
                    current_index_of_reference,
                    reference_path.display(),
                    reference.get_total_length(),
                    reference.get_num_targets(),
                );
                let out_file = File::create(reference_path)?;
                reference.save_to(out_file)?;
                current_index_of_reference += 1;
            }
        }
        // Save last reference
        let reference_path = reference_path_detector.get_path_of_index(current_index_of_reference);
        let reference = get_reference_with_default_option(sequence_storage)?;
        reference.get_total_length();
        num_targets_in_references.push(reference.get_num_targets());

        eprintln!(
            "Saving reference chunk index {} to {} ({} bp, {} targets)",
            current_index_of_reference,
            reference_path.display(),
            reference.get_total_length(),
            reference.get_num_targets(),
        );
        let out_file = File::create(reference_path)?;

        write_target_manifest_file(
            &num_targets_in_input_files,
            &num_targets_in_references,
            self.input_files.as_slice(),
            &reference_path_detector,
        )?;
        eprintln!("Target manifest file saved to {}", reference_path_detector.get_manifest_file_path().display());
        reference.save_to(out_file)?;
        
        Ok(())
    }
}

fn write_target_manifest_file(
    num_targets_in_input_files: &[u32],
    num_targets_in_references: &[u32],
    input_files: &[PathBuf],
    reference_path_detector: &ReferencePathDetector,
) -> Result<()> {
    let mut manifest_file_path = File::create(reference_path_detector.get_manifest_file_path())?;
    let mut writer = BufWriter::new(&mut manifest_file_path);

    let iter_1 = num_targets_in_input_files.iter().enumerate()
        .map(|(file_index, record_count)| {
            (0..*record_count).map(move |record_index| (file_index, record_index))
        }).flatten();
    let iter_2 = num_targets_in_references.iter().enumerate()
        .map(|(reference_index, target_count)| {
            (0..*target_count).map(move |record_index| (reference_index, record_index))
        }).flatten();
    writer.write_all("file_index\tfile_name\trecord_index\treference_index\ttarget_index\n".as_bytes()).unwrap();
    iter_1.zip(iter_2).for_each(|((file_index, record_index), (reference_index, target_index))| {
        let file_name = input_files[file_index].file_name().unwrap_or_default().to_str().unwrap_or_default();
        let line = format!(
            "{}\t{}\t{}\t{}\t{}\n",
            file_index, file_name, record_index, reference_index, target_index,
        );
        writer.write_all(line.as_bytes()).unwrap();
    });
    writer.flush()?;
    
    Ok(())
}

#[cfg(feature = "safe_guard")]
const USE_SAFE_GUARD: bool = true;
#[cfg(not(feature = "safe_guard"))]
const USE_SAFE_GUARD: bool = false;

fn get_reference_with_default_option(
    mut sequence_storage: InMemoryStorage,
) -> Result<Reference> {
    // Preparing sequence storage
    sequence_storage.set_sequences_to_uppercase();
    sequence_storage.change_bases_to(b"N", b'?');

    let total_length = sequence_storage.get_total_length();
    // Use half of total length as the maximum size of lookup table.
    // Maximum: 200 MiB
    let lookup_table_max_bytes_size = u64::min(
        200 * 1024 * 1024,
        (total_length / 8) as u64,
    );
    let dynamic_lfi_option = DynamicLfiOption {
        suffix_array_sampling_ratio: 1,
        lookup_table_max_bytes_size,
        use_safe_guard: USE_SAFE_GUARD,
    };
    let raw_reference = RawReference::new(
        sequence_storage,
        dynamic_lfi_option,
    )?;
    let reference = Reference::from_raw(raw_reference);
    Ok(reference)
}

fn input_file_extension_is_allowed(path: &PathBuf) -> bool {
    if is_gzip_file(path) {
        let path_without_gz = path.with_extension("");
        if is_fasta_file(path_without_gz) {
            return true
        } else {
            return false
        }
    } else if is_fasta_file(path) {
        return true
    } else {
        return false
    }
}
