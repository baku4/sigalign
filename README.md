# sigalign
Similarity Guided Alignment Algorithm
---
# Quick Start
```rust
use sigalign::{Reference, Aligner};
use sigalign::basic_sequence_provider::InMemoryProvider;

// (1) Make `Reference`
let mut sequence_provider = InMemoryProvider::new_empty();
sequence_provider.add_labeled_sequence(
    "record_1".to_string(),
    b"ATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCAAGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCATCGTATGCAGCACATAAAAATGCCAGAACCA".to_vec(),
);
sequence_provider.add_labeled_sequence(
    "record_2".to_string(),
    b"TTCCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCACCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGCGTATGCAGCACATAAAAAT".to_vec(),
);
let mut reference = Reference::new_with_default_config(sequence_provider).unwrap();

// (2) Make `Aligner`
let aligner = Aligner::new(4, 6, 2, 100, 0.1).unwrap();

// (3) Alignment with query
let query = b"TTCCTCTGTCATCAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGATAACGGAAGCACACCGATCTTAACCGGAGGTGCCGGATGAAAATGGAAAACGGTTCTTACGTCCGGCTTTTCCTCTGTTCCGATATTTTCCTCAT";
// - Semi-global alignment
let result_semi_global: String = aligner.semi_global_alignment_labeled(&mut reference, query).unwrap();
// - Local alignment
let result_local: String = aligner.local_alignment_labeled(&mut reference, query).unwrap();
```
# What is Sigalign?
 - Sigalign is pairwise alignment algorithm using gap-affine penalty for nucleotide and amino-acid sequences.
## Features
 - Sigalign has two values that evaluate the similarity between sequences.
   - (1) Aligned length
   - (2) Alignment penalty per aligned length
 - Sigalign returns the result of sequences satisfying the similarity criterion above **without exception**.
 - Sigalign has **only five parameters** that affect the alignment result.
   - Three are penalty values for mix-match, gap-open, and gap-extension.
   - Two are similarity values, which are aligned length and alignment penalty per aligned length.

Semi-global alignment
Local alignment
