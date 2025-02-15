use log::{error, info};
use crate::common::{
    init_logger,
    configuration::TestSetting,
    test_data::DataForValidation,
    random_regulator::gen_random_regulator,
};
use sigalign_utils::sequence_reader::{
    SeqRecord,
    SeqRefRecord,
    IdRecord,
    IdRefRecord,
    fasta::{FastaReader, FastaRecord},
    fastq::{FastqReader, FastqRecord},
};
use sigalign::{
    Aligner, Reference, ReferenceBuilder,
    algorithms::{Algorithm, Local, SemiGlobal},
    results::{Alignment, QueryAlignment, TargetAlignment},
    utils::formatter::SamWriter,
};

#[test]
fn print_results_as_sam() {
    // Get test data
    let test_data = DataForValidation::OneData;
    let (ref_file, qry_file) = test_data.get_data_paths();

    // Build Reference
    let reference = ReferenceBuilder::new()
        .add_fasta_file(&ref_file).unwrap()
        .build().unwrap();
    let mut aligner = Aligner::new(
        Local::new(4, 6, 2, 50, 0.1).unwrap()
    );

    // Get the first query
    let mut fasta_reader = FastaReader::new(
        std::fs::File::open(&qry_file).unwrap()
    );
    let mut query_buffer = Vec::new();
    let mut query_labels = String::new();
    while let Some(mut record) = fasta_reader.next() {
        record.extend_seq_buf(&mut query_buffer);
        record.extend_id_string(&mut query_labels).unwrap();
        break;
    }

    // Align
    let query_alignment = aligner.align(&query_buffer, &reference);

    // Write record as SAM
    let sam_results = {
        let mut sam_results_buffer = Vec::new();
        let mut sam_writer = SamWriter::from_writer(&mut sam_results_buffer);
        sam_writer.write_header().unwrap();
        // (1) Raw results
        sam_writer.write_query_alignment(
            &query_alignment,
            &query_labels,
            true,
            &reference,
        ).unwrap();
        // (2) Treat as reverse
        sam_writer.write_query_alignment(
            &query_alignment,
            &query_labels,
            false,
            &reference,
        ).unwrap();
        // (3) Raw results again
        sam_writer.write_query_alignment(
            &query_alignment,
            &query_labels,
            true,
            &reference,
        ).unwrap();

        drop(sam_writer);

        String::from_utf8(sam_results_buffer).unwrap()
    };

    // Print
    println!("{}", sam_results);
}