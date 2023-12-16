use std::io::Read;

use sigalign_core::{
    reference::{Reference as RawReference, PatternIndex, SequenceStorage},
    aligner::Aligner as RawAligner,
};
use sigalign_utils::sequence_reader::{
    fasta::FastaReader,
    SeqRecord, IdRefRecord,
};
use sigalign_impl::sequence_storage::in_memory::InMemoryBuffer;
use super::Aligner;
use crate::Reference;
use crate::results::*;

impl RawAligner for Aligner {
    fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &RawReference<I, S>,
        sequence_buffer: &mut S::Buffer,
        sorted_target_indices: &[u32],
        query: &[u8],
    ) -> AlignmentResult {
        self.dynamic_aligner.alignment(
            reference,
            sequence_buffer,
            sorted_target_indices,
            query,
        )
    }
}

impl Aligner {
    /* For one query */
    /// Align a query to the reference with a sequence buffer.
    /// ⚠️ This is lowest-level executor for `Aligner`, assuming that users have already known about "sigalign-core" and "sigalign-impl" crates.
    pub fn align_query_with_sequence_buffer<Q>(&mut self, reference: &Reference, sequence_buffer: &mut InMemoryBuffer, query: Q) -> AlignmentResult
    where
        Q: AsRef<[u8]>,
    {
        self.dynamic_aligner.alignment(
            reference.as_ref(),
            sequence_buffer,
            reference.get_full_sorted_target_indices(),
            query.as_ref(),
        )
    }
    /// Align a query to the reference.
    pub fn align_query<Q>(&mut self, reference: &Reference, query: Q) -> AlignmentResult
    where
        Q: AsRef<[u8]>,
    {
        let mut sequence_buffer = reference.as_ref().get_sequence_buffer();
        self.dynamic_aligner.alignment(
            reference.as_ref(),
            &mut sequence_buffer,
            reference.get_full_sorted_target_indices(),
            query.as_ref(),
        )
    }
    /// Align a query to the reference and label the result.
    pub fn align_query_labeled<Q>(&mut self, reference: &Reference, query: Q) -> LabeledAlignmentResult where
        Q: AsRef<[u8]>,
    {
        let mut sequence_buffer = reference.as_ref().get_sequence_buffer();
        let alignment_result = self.dynamic_aligner.alignment(
            reference.as_ref(),
            &mut sequence_buffer,
            reference.get_full_sorted_target_indices(),
            query.as_ref(),
        );
        label_the_alignment_result(alignment_result, reference)
    }

    /* For multiple query */
    /// Align multiple queries to the reference.
    pub fn align_queries<Q, I>(&mut self, reference: &Reference, queries: Q) -> Vec<AlignmentResult>
    where
        Q: IntoIterator<Item = I>,
        I: AsRef<[u8]>,
    {
        let mut sequence_buffer = reference.as_ref().get_sequence_buffer();
        queries.into_iter().map(|query| {
            self.dynamic_aligner.alignment(
                reference.as_ref(),
                &mut sequence_buffer,
                reference.get_full_sorted_target_indices(),
                query.as_ref(),
            )
        }).collect()
    }
    /// Align multiple queries to the reference and label the results.
    pub fn align_queries_labeled<Q, I>(&mut self, reference: &Reference, queries: Q) -> Vec<LabeledAlignmentResult> where
        Q: IntoIterator<Item = I>,
        I: AsRef<[u8]>,
    {
        let mut sequence_buffer = reference.as_ref().get_sequence_buffer();
        queries.into_iter().map(|query| {
            let alignment_result = self.dynamic_aligner.alignment(
                reference.as_ref(),
                &mut sequence_buffer,
                reference.get_full_sorted_target_indices(),
                query.as_ref(),
            );
            label_the_alignment_result(alignment_result, reference)
        }).collect()
    }
    /* For fasta */
    /// Align a FASTA file (can be read from any `Read`) to the reference.
    pub fn align_fasta<R>(&mut self, reference: &Reference, fasta: R) -> FastaAlignmentResult where
        R: Read,
    {
        let mut sequence_buffer = reference.as_ref().get_sequence_buffer();
        let mut fasta_reader = FastaReader::new(fasta);
        let mut query_buffer = Vec::new();
        let mut read_alignment_results = Vec::new();
        while let Some(mut record) =  fasta_reader.next() {
            query_buffer.clear();
            record.extend_seq_buf(&mut query_buffer);

            let alignment_result = self.dynamic_aligner.alignment(
                reference.as_ref(),
                &mut sequence_buffer,
                reference.get_full_sorted_target_indices(),
                &query_buffer,
            );
            if alignment_result.count_alignments() != 0 {
                let labeled_alignment_result = label_the_alignment_result(alignment_result, reference);
                let read = record.id_str().unwrap_or_default().to_string();
                let read_alignment_result = ReadAlignmentResult {
                    read,
                    is_forward: true,
                    result: labeled_alignment_result,
                };
                read_alignment_results.push(read_alignment_result);
            }
        }
        FastaAlignmentResult(read_alignment_results)
    }
    /// Align a FASTA file (can be read from any `Read`) to the reference with reverse complementary.
    /// E.g. if the reference is `ATCG`, then `CGAT` will be aligned to the reference.
    /// The result will be labeled as `is_forward = false`.
    /// The position of the alignment result will be the position of the reverse complementary sequence.
    pub fn align_fasta_with_reverse_complementary<R>(&mut self, reference: &Reference, fasta: R) -> FastaAlignmentResult where
        R: Read,
    {
        let mut sequence_buffer = reference.as_ref().get_sequence_buffer();
        let mut fasta_reader = FastaReader::new(fasta);
        let mut query_buffer = Vec::new();
        let mut read_alignment_results = Vec::new();
        while let Some(mut record) =  fasta_reader.next() {
            query_buffer.clear();
            record.extend_seq_buf(&mut query_buffer);
            // Forward
            let alignment_result = self.dynamic_aligner.alignment(
                reference.as_ref(),
                &mut sequence_buffer,
                reference.get_full_sorted_target_indices(),
                &query_buffer,
            );
            if alignment_result.count_alignments() != 0 {
                let labeled_alignment_result = label_the_alignment_result(alignment_result, reference);
                let read = record.id_str().unwrap_or_default().to_string();
                let read_alignment_result = ReadAlignmentResult {
                    read,
                    is_forward: true,
                    result: labeled_alignment_result,
                };
                read_alignment_results.push(read_alignment_result);
            }
            // Reverse
            transform_query_to_reverse_complementary_query(&mut query_buffer);
            let alignment_result = self.dynamic_aligner.alignment(
                reference.as_ref(),
                &mut sequence_buffer,
                reference.get_full_sorted_target_indices(),
                &query_buffer,
            );
            if alignment_result.count_alignments() != 0 {
                let labeled_alignment_result = label_the_alignment_result(alignment_result, reference);
                let read = record.id_str().unwrap_or_default().to_string();
                let read_alignment_result = ReadAlignmentResult {
                    read,
                    is_forward: false,
                    result: labeled_alignment_result,
                };
                read_alignment_results.push(read_alignment_result);
            }
        }
        FastaAlignmentResult(read_alignment_results)
    }
}

/* For label the results */
#[inline(always)]
fn label_the_alignment_result (
    alignment_result: AlignmentResult,
    reference: &Reference,
) -> LabeledAlignmentResult {
    LabeledAlignmentResult(
        alignment_result.0.into_iter().map(
            |x| label_the_target_alignment_result(x, reference)
        ).collect()
    )
}
#[inline(always)]
fn label_the_target_alignment_result (
    target_result: TargetAlignmentResult,
    reference: &Reference,
) -> LabeledTargetAlignmentResult {
    let target_index = target_result.index;
    let label = reference.get_label(target_index).unwrap_or_default();
    LabeledTargetAlignmentResult {
        index: target_index,
        label,
        alignments: target_result.alignments,
    }
}

/* For reverse complementary sequence */
fn transform_query_to_reverse_complementary_query(query_buffer: &mut Vec<u8>) {
    query_buffer.reverse();
    query_buffer.iter_mut().for_each(|x| {
        *x = match x {
            b'A' => b'T',
            b'T' => b'A',
            b'G' => b'C',
            b'C' => b'G',
            _ => *x,
        }
    });
}
