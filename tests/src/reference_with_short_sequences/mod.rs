use crate::common::init_logger;
use sigalign::ReferenceBuilder;

#[test]
fn test_short_sequence_bug_reproduction() {
    init_logger();

    // Exact bug scenario from issue report
    // 22-byte sequence with ignore_base should not fail
    let reference = ReferenceBuilder::new()
        .ignore_base(b'N')
        .add_target("barcode1", b"AAAAAAAAAAAAAAAAAAAAAA")  // 22 A's
        .build();

    assert!(reference.is_ok(), "Failed with 22bp sequence: {:?}", reference.err());
}

#[test]
fn test_very_short_single_sequence() {
    init_logger();

    // 20bp sequence
    let reference = ReferenceBuilder::new()
        .add_target("short", b"ACGTACGTACGTACGTACGT")
        .build();

    assert!(reference.is_ok(), "Failed with 20bp sequence: {:?}", reference.err());
}

#[test]
fn test_multiple_short_sequences() {
    init_logger();

    // Multiple 30bp sequences
    let reference = ReferenceBuilder::new()
        .add_target("seq1", b"ACGTACGTACGTACGTACGTACGTACGTAC")
        .add_target("seq2", b"TGCATGCATGCATGCATGCATGCATGCATG")
        .build();

    assert!(reference.is_ok(), "Failed with multiple 30bp sequences: {:?}", reference.err());
}

#[test]
fn test_single_base_sequence() {
    init_logger();

    // Single base - extreme edge case
    let reference = ReferenceBuilder::new()
        .add_target("single", b"A")
        .build();

    assert!(reference.is_ok(), "Failed with single base: {:?}", reference.err());
}

#[test]
fn test_mixed_length_sequences() {
    init_logger();

    // Mixed lengths: 25bp, 100bp, 1000bp
    let reference = ReferenceBuilder::new()
        .add_target("short", b"ACGTACGTACGTACGTACGTACGTA")
        .add_target("medium", b"ACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGTACGT")
        .add_target("long", &b"ACGT".repeat(250))
        .build();

    assert!(reference.is_ok(), "Failed with mixed length sequences: {:?}", reference.err());
}

#[test]
fn test_alignment_with_short_reference() {
    init_logger();

    // Verify that short sequences can actually perform alignments
    let reference = ReferenceBuilder::new()
        .add_target("adapter", b"AGATCGGAAGAGCACACGTCT")  // 21bp Illumina adapter
        .build()
        .expect("Failed to build reference with short sequence");

    // Try to create an aligner - this should work if the fix is correct
    use sigalign::{Aligner, algorithms::Local};

    let px = 4;
    let po = 6;
    let pe = 2;
    let minl = 10;
    let maxp = 0.1;

    let algorithm = Local::new(px, po, pe, minl, maxp).expect("Failed to create algorithm");
    let mut aligner = Aligner::new(algorithm);

    // Test alignment with exact match
    let query = b"AGATCGGAAGAGCACACGTCT";
    let result = aligner.align(query, &reference);

    assert!(!result.0.is_empty(), "Should find alignment for exact match");
}
