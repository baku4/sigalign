use std::path::Path;

const FASTA_EXTENSIONS: [&str; 6] = [
    "fa", "fasta", // General extension
    "fna", //For nucleotide
    "ffn", // For nucleotide of coding regions (genes)
    "faa", //For amino acid (protein)
    "frn", // For non-coding RNA sequences
];

const GZIP_EXTENSIONS: [&str; 2] = [
    "gz", "gzip",
];

pub fn is_fasta_file<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    let extension = path.extension().unwrap_or_default();
    FASTA_EXTENSIONS.contains(&extension.to_str().unwrap_or_default())
}

pub fn is_gzip_file<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    let extension = path.extension().unwrap_or_default();
    GZIP_EXTENSIONS.contains(&extension.to_str().unwrap_or_default())
}
