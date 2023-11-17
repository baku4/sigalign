pub use anyhow::{Result, bail as error_msg};

// Allowed input file extensions
// - FASTA (.fa, .fasta, .fna, .ffn, .faa, .frn)
// - gzip compressed FASTA
// - bzip2 compressed FASTA
// - zip compressed FASTA
const ALLOWED_INPUT_FILE_EXTENSIONS: [&str; 10] = [
    "fa", "fasta", "fna", "ffn", "faa", "frn", "gz", "bz2", "zip", "tar",
];