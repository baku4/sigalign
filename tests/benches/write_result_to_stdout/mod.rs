use std::io::{Write, BufWriter, StdoutLock};

use criterion::{
    black_box, Criterion, BenchmarkId,
    PlotConfiguration, AxisScale,
};
use sigalign_tests::get_stable_result_of_val_data;
use sigalign::results::{
    fasta::{
        FastaAlignmentResult,
        ReadAlignmentResult,
    },
    AlignmentResult,
    TargetAlignmentResult,
    AnchorAlignmentResult,
    AlignmentOperations,
    AlignmentOperation,
};

fn get_test_result() -> FastaAlignmentResult {
    get_stable_result_of_val_data()
}

// Bench main

pub fn write_result_to_stdout(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_result_to_stdout");
    let test_result = get_test_result();

    group.bench_function(
        "unlocked_stdout",
        |b| b.iter(|| {
            unlocked_stdout(black_box(&test_result));
        }
    ));
    group.bench_function(
        "locked_stdout",
        |b| b.iter(|| {
            locked_stdout(black_box(&test_result));
        }
    ));
    group.bench_function(
        "use_bufwriter",
        |b| b.iter(|| {
            use_bufwriter(black_box(&test_result));
        }
    ));
    group.bench_function(
        "use_bufwriter_directly_with_itoa",
        |b| b.iter(|| {
            use_bufwriter_directly_with_itoa(black_box(&test_result));
        }
    ));
    group.bench_function(
        "use_bufwriter_directly_ops_with_itoa",
        |b| b.iter(|| {
            use_bufwriter_directly_ops_with_itoa(black_box(&test_result));
        }
    ));
}

// Each function to bench

#[inline]
fn unlocked_stdout(result: &FastaAlignmentResult) {
    let mut writer = std::io::stdout();
    result.0.iter().for_each(|ReadAlignmentResult {
        read: label,
        result: alignment_result
    }| {
        alignment_result.0.iter().for_each(|TargetAlignmentResult {
            index: target_index,
            alignments: anchor_results,
        }| {
            anchor_results.iter().for_each(|anchor_result| {
                let line = format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    label, target_index, anchor_result.penalty, anchor_result.length,
                    anchor_result.position.query.0, anchor_result.position.query.1,
                    anchor_result.position.target.0, anchor_result.position.target.1,
                    operations_to_string(&anchor_result.operations)
                ).into_bytes();
                let _ = writer.write(&line).unwrap();
            });
        });
    });
}
#[inline]
fn locked_stdout(result: &FastaAlignmentResult) {
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    result.0.iter().for_each(|ReadAlignmentResult {
        read: label,
        result: alignment_result
    }| {
        alignment_result.0.iter().for_each(|TargetAlignmentResult {
            index: target_index,
            alignments: anchor_results,
        }| {
            anchor_results.iter().for_each(|anchor_result| {
                let line = format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    label, target_index, anchor_result.penalty, anchor_result.length,
                    anchor_result.position.query.0, anchor_result.position.query.1,
                    anchor_result.position.target.0, anchor_result.position.target.1,
                    operations_to_string(&anchor_result.operations)
                ).into_bytes();
                let _ = writer.write(&line).unwrap();
            });
        });
    });
}
#[inline]
fn use_bufwriter(result: &FastaAlignmentResult) {
    let stdout = std::io::stdout();
    let lock = stdout.lock();
    let mut writer = std::io::BufWriter::with_capacity(
        32 * 1024,
        lock,
    );
    
    result.0.iter().for_each(|ReadAlignmentResult {
        read: label,
        result: alignment_result
    }| {
        alignment_result.0.iter().for_each(|TargetAlignmentResult {
            index: target_index,
            alignments: anchor_results,
        }| {
            anchor_results.iter().for_each(|anchor_result| {
                let line = format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    label, target_index, anchor_result.penalty, anchor_result.length,
                    anchor_result.position.query.0, anchor_result.position.query.1,
                    anchor_result.position.target.0, anchor_result.position.target.1,
                    operations_to_string(&anchor_result.operations)
                ).into_bytes();
                let _ = writer.write(&line).unwrap();
            });
        });
    });
}
#[inline]
fn use_bufwriter_directly_with_itoa(result: &FastaAlignmentResult) {
    let stdout = std::io::stdout();
    let lock = stdout.lock();
    let mut writer = std::io::BufWriter::with_capacity(
        32 * 1024,
        lock,
    );
    let mut itoa_buffer = itoa::Buffer::new();
    
    result.0.iter().for_each(|ReadAlignmentResult {
        read: label,
        result: alignment_result
    }| {
        alignment_result.0.iter().for_each(|TargetAlignmentResult {
            index: target_index,
            alignments: anchor_results,
        }| {
            anchor_results.iter().for_each(|anchor_result| {
                let _ = writer.write(label.as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(*target_index).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.penalty).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.length).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.position.query.0).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.position.query.1).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.position.target.0).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.position.target.1).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(operations_to_string(&anchor_result.operations).as_bytes()).unwrap();
                let _ = writer.write(b"\n").unwrap();
            });
        });
    });
}
#[inline]
fn use_bufwriter_directly_ops_with_itoa(result: &FastaAlignmentResult) {
    let stdout = std::io::stdout();
    let lock = stdout.lock();
    let mut writer = std::io::BufWriter::with_capacity(
        32 * 1024,
        lock,
    );
    let mut itoa_buffer = itoa::Buffer::new();
    
    result.0.iter().for_each(|ReadAlignmentResult {
        read: label,
        result: alignment_result
    }| {
        alignment_result.0.iter().for_each(|TargetAlignmentResult {
            index: target_index,
            alignments: anchor_results,
        }| {
            anchor_results.iter().for_each(|anchor_result| {
                let _ = writer.write(label.as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(*target_index).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.penalty).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.length).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.position.query.0).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.position.query.1).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.position.target.0).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                let _ = writer.write(itoa_buffer.format(anchor_result.position.target.1).as_bytes()).unwrap();
                let _ = writer.write(b"\t").unwrap();
                anchor_result.operations.iter().for_each(|AlignmentOperations {
                    operation,
                    count,
                }| {
                    let _ = writer.write(
                        match operation {
                            AlignmentOperation::Match => b"M",
                            AlignmentOperation::Subst => b"S",
                            AlignmentOperation::Insertion => b"I",
                            AlignmentOperation::Deletion => b"D",
                        }
                    ).unwrap();
                    let _ = writer.write(itoa_buffer.format(*count).as_bytes()).unwrap();
                });
                let _ = writer.write(b"\n").unwrap();
            });
        });
    });
}

// Operation

#[inline]
fn operations_to_string(operations: &Vec<AlignmentOperations>) -> String {
    let string_ops: Vec<String> = operations.iter().map(|op| {
        format!(
            "{}{}",
            match op.operation {
                AlignmentOperation::Match => 'M',
                AlignmentOperation::Subst => 'S',
                AlignmentOperation::Insertion => 'I',
                AlignmentOperation::Deletion => 'D',
            },
            op.count,
        )
    }).collect();
    string_ops.concat()
}
