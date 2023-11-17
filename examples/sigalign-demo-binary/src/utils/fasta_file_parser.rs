// Allowed input file extensions
// - FASTA (.fa, .fasta, .fna, .ffn, .faa, .frn)
pub const FASTA_FILE_EXTENSIONS: [&str; 6] = [
    "fa", "fasta", "fna", "ffn", "faa", "frn",
];

// Allowed compressed file extensions
// - gzip compressed FASTA
// TODO:
// - bzip2 compressed FASTA
// - zip compressed FASTA
// - tar compressed FASTA
// - tar.gz compressed FASTA
// - tar.bz2 compressed FASTA
pub const COMPRESSED_FILE_EXTENSIONS: [&str; 1] = [
    "gz", // "bz2", "zip", "tar", "tar.gz", "tar.bz2",
];
