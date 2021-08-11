use criterion::{
    black_box, criterion_group, criterion_main, Criterion, BenchmarkId
};
use lt_fm_index::*;


/*
New anchor vs Old anchor
*/
use dropout_pairwise::*;
use dropout_pairwise::reference::*;
use dropout_pairwise::alignment::{*, anchor_dep, anchor};

fn bench_new_vs_old_anchor(c: &mut Criterion) {
    let ref_seq = b"CTCCGTGATTTACCGGATGCATAGATTTCCCCATTTTGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAACACCTGTTTCGTATCGGAACCGGTAAGTGAAATTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTTCCGTACACCTGTTTCGTATCGGAACCGGTAAGTGAAATTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTGGATCCGGCTCCTGCGTGGAAAACCAGTCATCCTGATTTACATATGGTTCAATGGCACCGGATTGTTGCTGGTTCAATGGCACCGGATGCATAGATTTCCCCATTTTGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGATTTACCGGATGCATAGATTTCCCCATTTTGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAGTATCGGAACCGGTAAGTGAAATTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTGGATCCGGCTCCTGCGTGGAAAACCAGTCATCCTGATTTACATATGGTTTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTGGATCCGGCTCCTGCGTGGAAAACCAGTCATCCTGATTTACATATGGTTCAATGGCACCGGATGCATAGATTTCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAGTGATTTACCGGATGCATAGATTTCCCCATTTTGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAAAATTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTGGATCCG".to_vec();

    let qry_seq = b"AAACCGTATATTGTCCATCCGCTGCCGGTGGATCCGGCTCCTGCGTGGAAAACCAGTCATCCTGATTTACATATGGTTTCCACATCGCCGGAAACCGTAAAATGTCCATCCGCTGCCGGTGGATCCGGCTCCTGCGTGGAAAACCAGTCATCCTGATTTACATATGGTTCAATGGCACCGGATGCATAGATTTCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAGTGATTTACCGGATGCATAGATTTCCCCATTTTGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACCGGTAAAAATTTCCACATCGCCGGAAACCGTATATTGTCCATCCGCTGCCGGTGGATCCGTGCATAGATTTAAACCGGAAACGTGCGCAAGCACGATGTGTCTTACCCTCCGTACACCTGTTTCGTATCGGAACGTAAGTGATTTACCGGATGCATAGATTTCCCCAGCGTACCGGAAACGTGCGCAAGCACGATCTGTGTCTTACCCTCCGTAC".to_vec();

    let kmer_klt = 8;
    let ssr = 2;
    let ml = 30;
    let ppl = 0.1;
    let x = 4;
    let o = 3;
    let e = 2;
    // for new anchor
    let cutoff = Cutoff::new(ml, ppl);
    let penalties = Penalties::new(x,o,e);
    let reference = ReferenceConfig::new()
        .contain_only_nucleotide(true)
        // .search_reverse_complement(true)
        .set_kmer_size_for_klt(kmer_klt)
        .set_sampling_ratio_for_sa(ssr)
        .generate_reference_with_string(ref_seq.clone());
    let aligner = Aligner::new(cutoff, penalties, reference);

    // for old anchor
    let fm_index_1 = FmIndexConfig::new()
        .set_kmer_lookup_table(kmer_klt)
        .set_suffix_array_sampling_ratio(ssr)
        .generate_fmindex(ref_seq.clone());

    
    let mut anchor_group_new = anchor::AnchorsGroup::new(
        black_box(&aligner.penalties),
        black_box(&aligner.cutoff),
        black_box(&aligner.block_penalty),
        black_box(&aligner.reference),
        black_box(aligner.kmer),
        black_box(&qry_seq)
    );
    let mut anchor_group_old_1 = anchor_dep::AnchorGroup::new(
        black_box(&ref_seq),
        black_box(&qry_seq),
        black_box(&fm_index_1),
        black_box(aligner.kmer),
        black_box(&aligner.block_penalty),
        black_box(&aligner.penalties),
        black_box(&aligner.cutoff)
    );

    let mut group = c.benchmark_group("gen_new_vs_old_anchor_one");

    group.bench_function(
        "NewAnchor",
        |b| b.iter(|| {
            let mut anchor_group_new = anchor::AnchorsGroup::new(
                black_box(&aligner.penalties),
                black_box(&aligner.cutoff),
                black_box(&aligner.block_penalty),
                black_box(&aligner.reference),
                black_box(aligner.kmer),
                black_box(&qry_seq)
            );
        }
    ));
    group.bench_function(
        "OldAnchor",
        |b| b.iter(|| {
            let mut anchor_group_old_1 = anchor_dep::AnchorGroup::new(
                black_box(&ref_seq),
                black_box(&qry_seq),
                black_box(&fm_index_1),
                black_box(aligner.kmer),
                black_box(&aligner.block_penalty),
                black_box(&aligner.penalties),
                black_box(&aligner.cutoff)
            );
        }
    ));

    group.finish();

    let mut group = c.benchmark_group("align_new_vs_old_anchor_one");

    group.bench_function(
        "NewAnchor",
        |b| b.iter(|| {
            let res = anchor_group_new.alignment(
                black_box(&aligner.penalties),
                black_box(&aligner.cutoff),
                black_box(&aligner.reference),
                black_box(&qry_seq),
                black_box(false)
            );
        }
    ));
    group.bench_function(
        "OldAnchor",
        |b| b.iter(|| {
            let res = if let Some(ag) = &mut anchor_group_old_1 {
                ag.alignment(black_box(false));
                ag.get_result(black_box(false))
            } else {
                Vec::new()
            };
        }
    ));

    group.finish();
}

criterion_group!(
    benches,
    bench_new_vs_old_anchor,
);
criterion_main!(benches);