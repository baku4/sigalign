use std::io::{Write, BufWriter, StdoutLock};

use criterion::{
    black_box, Criterion, BenchmarkId,
    PlotConfiguration, AxisScale,
};

use sigalign::results::LabeledQueryAlignment;
use sigalign_tests::common::test_alignment_results::get_test_alignment_results;

// Bench main

pub fn convert_results_to_tsv_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("convert_results_to_tsv_string");
    let test_results = get_test_alignment_results().unwrap();

    group.bench_function(
        "with_only_format_macro_using_iter",
        |b| b.iter(|| {
            with_only_format_macro_using_iter(black_box(&test_results));
        }
    ));
    group.bench_function(
        "with_only_format_macro_using_push",
        |b| b.iter(|| {
            with_only_format_macro_using_push(black_box(&test_results));
        }
    ));
    group.bench_function(
        "with_itoa_with_bytes",
        |b| b.iter(|| {
            with_itoa_with_bytes(black_box(&test_results));
        }
    ));
}

// Each function to bench

#[inline(always)]
fn with_only_format_macro_using_iter(results: &Vec<(String, LabeledQueryAlignment)>) -> String {
    results.iter().map(|(query_name, labeled_query_alignment)| {
        labeled_query_alignment.0.iter().map(move |labeled_target_alignment| {
            labeled_target_alignment.alignments.iter().map(move |alignment| {
                format!(
                    "{}\tF\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    query_name,
                    labeled_target_alignment.label,
                    alignment.penalty,
                    alignment.length,
                    alignment.position.query.0,
                    alignment.position.query.1,
                    alignment.position.target.0,
                    alignment.position.target.1,
                    alignment.operations.iter().map(|op| {
                        format!("{}{}", op.count, match op.operation {
                            sigalign::results::AlignmentOperation::Match => "=",
                            sigalign::results::AlignmentOperation::Subst => "X",
                            sigalign::results::AlignmentOperation::Insertion => "I",
                            sigalign::results::AlignmentOperation::Deletion => "D",
                        })
                    }).collect::<Vec<String>>().join("")
                )
            })
        }).flatten()
    }).flatten().collect()
}

#[inline(always)]
fn with_only_format_macro_using_push(results: &Vec<(String, LabeledQueryAlignment)>) -> String {
    let mut string_result = String::new();

    results.iter().for_each(|(query_name, labeled_query_alignment)| {
        labeled_query_alignment.0.iter().for_each(|labeled_target_alignment| {
            labeled_target_alignment.alignments.iter().for_each(|alignment| {
                string_result.push_str(query_name);
                string_result.push_str("\tF\t");
                string_result.push_str(&labeled_target_alignment.label);
                string_result.push_str("\t");
                string_result.push_str(&alignment.penalty.to_string());
                string_result.push_str("\t");
                string_result.push_str(&alignment.length.to_string());
                string_result.push_str("\t");
                string_result.push_str(&alignment.position.query.0.to_string());
                string_result.push_str("\t");
                string_result.push_str(&alignment.position.query.1.to_string());
                string_result.push_str("\t");
                string_result.push_str(&alignment.position.target.0.to_string());
                string_result.push_str("\t");
                string_result.push_str(&alignment.position.target.1.to_string());   
                string_result.push_str("\t");
                string_result.push_str(&alignment.operations.iter().map(|op| {
                    format!("{}{}", op.count, match op.operation {
                        sigalign::results::AlignmentOperation::Match => "=",
                        sigalign::results::AlignmentOperation::Subst => "X",
                        sigalign::results::AlignmentOperation::Insertion => "I",
                        sigalign::results::AlignmentOperation::Deletion => "D",
                    })
                }).collect::<Vec<String>>().join(""));
                string_result.push_str("\n");
            })
        })
    });

    string_result
}

#[inline(always)]
fn with_itoa_with_bytes(results: &Vec<(String, LabeledQueryAlignment)>) -> String {
    let mut itoa_buffer = itoa::Buffer::new();
    let mut bytes_results = Vec::new();

    results.iter().for_each(|(query_name, labeled_query_alignment)| {
        labeled_query_alignment.0.iter().for_each(|labeled_target_alignment| {
            labeled_target_alignment.alignments.iter().for_each(|alignment| {
                bytes_results.extend(query_name.as_bytes());
                bytes_results.extend(b"\tF\t");
                bytes_results.extend(labeled_target_alignment.label.as_bytes());
                bytes_results.extend(b"\t");
                bytes_results.extend(itoa_buffer.format(alignment.penalty).as_bytes());
                bytes_results.extend(b"\t");
                bytes_results.extend(itoa_buffer.format(alignment.length).as_bytes());
                bytes_results.extend(b"\t");
                bytes_results.extend(itoa_buffer.format(alignment.position.query.0).as_bytes());
                bytes_results.extend(b"\t");
                bytes_results.extend(itoa_buffer.format(alignment.position.query.1).as_bytes());
                bytes_results.extend(b"\t");
                bytes_results.extend(itoa_buffer.format(alignment.position.target.0).as_bytes());
                bytes_results.extend(b"\t");
                bytes_results.extend(itoa_buffer.format(alignment.position.target.1).as_bytes());
                bytes_results.extend(b"\t");
                alignment.operations.iter().for_each(|op| {
                    bytes_results.extend(itoa_buffer.format(op.count).as_bytes());
                    bytes_results.extend(match op.operation {
                        sigalign::results::AlignmentOperation::Match => b"=",
                        sigalign::results::AlignmentOperation::Subst => b"X",
                        sigalign::results::AlignmentOperation::Insertion => b"I",
                        sigalign::results::AlignmentOperation::Deletion => b"D",
                    });
                });
                bytes_results.extend(b"\n");
            })
        })
    });

    String::from_utf8(bytes_results).unwrap()
}
