use super::test_data::DataForValidation;
use anyhow::Result;

use sigalign::{algorithms::Local, results::LabeledQueryAlignment, Aligner, ReferenceBuilder};
use sigalign_utils::sequence_reader::{fasta::FastaReader, SeqRecord, IdRecord};
use sigalign_core::reference;

pub fn get_test_alignment_results() -> Result<Vec<(String, LabeledQueryAlignment)>> {
    let (ref_file, qry_file) = DataForValidation::Default.get_data_paths();
    let reference = ReferenceBuilder::new()
        .add_fasta_file(ref_file)?
        .build()?;

    let mut aligner = Aligner::new(
        Local::new(4, 6, 2, 50, 0.05)?
    );

    let mut query = Vec::new();
    let mut label = String::new();
    let mut fasta_reader = FastaReader::from_path(qry_file)?;

    let mut results = Vec::new();
    while let Some(mut record) = fasta_reader.next() {
        query.clear();
        label.clear();
        record.extend_seq_buf(&mut query);
        record.extend_id_string(&mut label)?;

        let result = aligner.align(&query, &reference);
        let result = reference.label_query_alignment(result);

        results.push((label.clone(), result));
    }
    Ok(results)
}