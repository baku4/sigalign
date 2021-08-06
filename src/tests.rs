mod data;
mod classic_wfa;
mod dp_pairwise;
mod multi_thread;

use crate::{*, alignment::{Cutoff, Penalties}};

#[derive(Debug, Clone, Copy)]
struct AlignmentOption {
    // cutoff
    score_per_length: f64,
    minimum_length: SequenceLength,
    // penalty
    mismatch_penalty: Penalty,
    gapopen_penalty: Penalty,
    gapext_penalty: Penalty,
    // other options for dropout pairwise
    using_cached_wf: bool,
    get_minimum_penalty: bool,
}

fn alignment_using_dp(
    alignment_option: AlignmentOption, ref_seq: Vec<u8>, qry_seq: Vec<u8>
) -> Option<alignment::AlignmentResult> {
    let aligner = dp_pairwise::DpAligner::new(alignment_option.score_per_length, alignment_option.minimum_length, alignment_option.mismatch_penalty, alignment_option.gapopen_penalty, alignment_option.gapext_penalty);
    let res = dp_pairwise::alignment(&aligner, &ref_seq, &qry_seq);
    if res.len() == 0 {
        None
    } else {
        Some(res)
    }
}

fn alignment_using_dwfa(
    alignment_option: AlignmentOption, ref_seq: Vec<u8>, qry_seq: Vec<u8>
) -> Option<alignment::AlignmentResult> {
    let cutoff = Cutoff::new(alignment_option.minimum_length, alignment_option.score_per_length);
    let penalties = Penalties::new(alignment_option.mismatch_penalty, alignment_option.gapopen_penalty, alignment_option.gapext_penalty);
    let aligner = alignment::Aligner::new(cutoff, penalties);
    // FIXME:
    // let res= aligner.align_with_only_sequences(&ref_seq, &qry_seq);
    //
    None
}

mod compare_result {
    use super::*;
    use crate::*;
    use std::time::{Duration, Instant};

    #[allow(dead_code)]
    #[test]
    fn test_alignment_result() {
        // setting option
        let alignment_option = AlignmentOption {
            score_per_length: 0.1,
            minimum_length: 100,
            mismatch_penalty: 4,
            gapopen_penalty: 3,
            gapext_penalty: 2,
            using_cached_wf: true,
            get_minimum_penalty: true,
        };
        // executor
        let mut executor = multi_thread::print_output::Executor::new(1);
        let job_sender = executor.get_sender().unwrap();
        // get 
        let map = data::get_connected_map();
        // read data
        let mut ref_records = data::ref_fasta_records();
        while let Some(Ok(ref_record)) = ref_records.next() {
            let matched_qry_id = if let Some(qry_string) = map.get(ref_record.id()) {
                qry_string.clone()
            } else {
                continue;
            };
            let mut qry_records = data::qry_fasta_records();
            while let Some(Ok(qry_record)) = qry_records.next() {
                if matched_qry_id != qry_record.id() {
                    continue;
                }
                // setting job 1
                let print_using_dp = |label: String| {
                    move |alignment_option: AlignmentOption, ref_seq: Vec<u8>, qry_seq: Vec<u8>| {
                        let res = alignment_using_dp(alignment_option, ref_seq, qry_seq);
                        if let Some(v) = res {
                            println!("{}\n{:?}", label, v);
                        }
                    }
                };
                let boxed_function_dp: Box::<dyn FnOnce(AlignmentOption, Vec<u8>, Vec<u8>) + Send + 'static> = Box::new(print_using_dp(format!(
                    "#DP,{},{}", ref_record.id().to_string(), qry_record.id().to_string()
                )));
                let job_1: multi_thread::print_output::Job = (
                    alignment_option.clone(),
                    ref_record.seq().to_vec(),
                    qry_record.seq().to_vec(),
                    boxed_function_dp,
                );
                // setting job 2
                let print_using_dropout = |label: String| {
                    move |alignment_option: AlignmentOption, ref_seq: Vec<u8>, qry_seq: Vec<u8>| {
                        let res = alignment_using_dwfa(alignment_option, ref_seq, qry_seq);
                        if let Some(v) = res {
                            println!("{}\n{:?}", label, v);
                        }
                    }
                };
                let boxed_function_dropout: Box::<dyn FnOnce(AlignmentOption, Vec<u8>, Vec<u8>) + Send + 'static> = Box::new(print_using_dropout(format!(
                    "#DROPOUT,{},{}", ref_record.id().to_string(), qry_record.id().to_string()
                )));
                let job_2: multi_thread::print_output::Job = (
                    alignment_option.clone(),
                    ref_record.seq().to_vec(),
                    qry_record.seq().to_vec(),
                    boxed_function_dropout,
                );
                // Send job
                let _ = job_sender.send(job_1);
                let _ = job_sender.send(job_2);
            }
        }
    }

    #[test]
    fn print_alignment_time() {
        // setting option
        let alignment_option = AlignmentOption {
            score_per_length: 0.1,
            minimum_length: 100,
            mismatch_penalty: 4,
            gapopen_penalty: 3,
            gapext_penalty: 2,
            using_cached_wf: true,
            get_minimum_penalty: true,
        };
        // executor
        let mut executor = multi_thread::print_output::Executor::new(4);
        let job_sender = executor.get_sender().unwrap();
        // read data
        let mut ref_records = data::ref_fasta_records();
        while let Some(Ok(ref_record)) = ref_records.next() {
            let mut qry_records = data::qry_fasta_records();
            while let Some(Ok(qry_record)) = qry_records.next() {
                // setting job
                let compare_time = |label: String| {
                    move |alignment_option: AlignmentOption, ref_seq: Vec<u8>, qry_seq: Vec<u8>| {
                        // (1) using dp
                        let time_1 = {
                            let start = Instant::now();
                            let _ = alignment_using_dp(alignment_option, ref_seq.clone(), qry_seq.clone());
                            let duration = start.elapsed();
                            duration.as_micros()
                        };
                        // (2) using new anchor
                        let time_2 = {
                            let start = Instant::now();
                            let _ = alignment_using_dwfa(alignment_option, ref_seq.clone(), qry_seq.clone());
                            let duration = start.elapsed();
                            duration.as_micros()
                        };
                        // print time
                        println!("{},{},{}", label, time_1, time_2);
                    }
                };
                let boxed_function: Box::<dyn FnOnce(AlignmentOption, Vec<u8>, Vec<u8>) + Send + 'static> = Box::new(
                    compare_time(format!(
                        "{}:{}", ref_record.id().to_string(), qry_record.id().to_string()
                    ))
                );

                let job: multi_thread::print_output::Job = (
                    alignment_option.clone(),
                    ref_record.seq().to_vec(),
                    qry_record.seq().to_vec(),
                    boxed_function,
                );
                // Send job
                let _ = job_sender.send(job);
            }
        }
    }
}