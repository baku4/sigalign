use super::*;
use crate::test_data::*;
use std::{path::PathBuf, ops::Range, io::{Read, Write}};
use ahash::{AHashMap, AHashSet};

type DefaultReference = Reference<InMemoryRcStorage>;

#[test]
fn print_debug_status_of_local_alignment_results() {
    init_logger();

    // (1) Build reference and aligners
    let reference = get_default_reference();
    let (dp_aligner, mut sig_aligner) = get_default_aligners(true);

    // (2) Make cache result with Dp Aligner
    let local_tmp_dir = get_local_tmp_dir().unwrap();

    let qry_file = get_qry_for_val_path();
    
    // Caching alignments
    let mut qry_reader = FastaReader::from_file_path(qry_file).unwrap();

    let qry_count = 200; // TODO: Use Total Qry

    for idx in 0..qry_count {
        let (label, query) = qry_reader.next().unwrap();
        info!("# Start idx: {}, label: {}", idx, label);

        let dp_res = dp_aligner.get_cached_result(
            true,
            &reference,
            &label,
            &query,
            &local_tmp_dir,
        );

        let sig_res = sig_aligner.query_alignment_unchecked(&reference, &query);
        info!("Result count: DP - {}, Sig - {}", dp_res.0.len(), sig_res.0.len());

        let (is_same, only_in_dp, only_in_sig) = compare_alignment_result(&dp_res, &sig_res);
        info!("Alignment is same?- {}", is_same);
        info!("only_in_dp- {:?}", only_in_dp);
        info!("only_in_sig- {:?}", only_in_sig);
    }
}

impl DpBasedAligner {
    fn get_cached_result(
        &self,
        is_local: bool,
        reference: &DefaultReference,
        qry_label: &String,
        qry_seq: &[u8],
        tmp_dir: &PathBuf,
    ) -> AlignmentResult {
        let file_name = format!(
            "DP_{}_{}_{}_{}_{}_{}_{}.cache",
            self.mismatch_penalty,
            self.gap_open_penalty,
            self.gap_extend_penalty,
            self.minimum_aligned_length,
            self.maximum_penalty_per_scale,
            is_local,
            qry_label,
        );
        let cache_file = tmp_dir.clone().join(file_name);
        let cached_result = Self::parse_cache(&cache_file);
        if cached_result.is_some() {
            info!("Alignment extracted from cache")
        }

        match cached_result {
            Some(v) => v,
            None => {
                let result = self.local_alignment(&reference, qry_seq);
                let json = result.to_json();
                let mut out_file = std::fs::File::create(cache_file).unwrap();
                out_file.write_all(json.as_bytes()).unwrap();

                result
            }
        }
    }
    fn parse_cache(cache_file: &PathBuf) -> Option<AlignmentResult> {
        if cache_file.exists() {
            let mut out_file = match std::fs::File::open(cache_file) {
                Ok(v) => v,
                Err(_) => return None,
            };

            let mut json = String::new();
            match out_file.read_to_string(&mut json) {
                Ok(_) => {},
                Err(_) => return None,
            };
            let result = match AlignmentResult::from_json(&json) {
                Ok(v) => v,
                Err(e) => {
                    error!("Extract json error: {}", e);
                    return None
                },
            };

            Some(result)
        } else {
            None
        }
    }
}

fn get_default_aligners(is_local: bool) -> (DpBasedAligner, Aligner) {
    let px = 4;
    let po = 6;
    let pe = 2;
    let mal = 50;
    let mppl = 0.1;

    let dp_based_aligner = DpBasedAligner::new(
        px,
        po,
        pe,
        mal,
        mppl,
    );
    let sig_aligner = if is_local {
        Aligner::new_local(
            px,
            po,
            pe,
            mal,
            mppl,
        ).unwrap()
    } else {
        Aligner::new_semi_global(
            px,
            po,
            pe,
            mal,
            mppl,
        ).unwrap()
    };

    (dp_based_aligner, sig_aligner)
}
fn get_default_reference() -> DefaultReference {
    let ref_file = get_ref_for_val_path();

    let mut sequence_storage = InMemoryRcStorage::new();
    sequence_storage.add_fasta_file(ref_file).unwrap();

    let reference = ReferenceBuilder::new()
        .change_bwt_block_size_to_128()
        .change_count_array_kmer(4).unwrap()
        .change_sampling_ratio(2).unwrap()
        .build(sequence_storage).unwrap();

    reference
}

type RecordAlignmentMap = AHashMap<usize, AHashSet<AnchorAlignmentResult>>;
fn compare_alignment_result(a: &AlignmentResult, b: &AlignmentResult) -> (bool, RecordAlignmentMap, RecordAlignmentMap) {
    // To hash map
    let to_map = |ar: &AlignmentResult| -> RecordAlignmentMap {
        ar.0.iter().map(|record_alignment_result| {
            (record_alignment_result.index, record_alignment_result.alignments.clone().into_iter().collect())
        }).collect()
    };
    let mut a_map = to_map(a);
    let mut b_map = to_map(b);

    // Compare record_alignment_result
    let mut is_same = true;

    let a_map_keys: AHashSet<usize> = a_map.keys().map(|v| *v).collect();
    let b_map_keys: AHashSet<usize> = b_map.keys().map(|v| *v).collect();

    let common_rec_indices = a_map_keys.intersection(&b_map_keys);
    for &common_rec_index in common_rec_indices {
        let a = a_map.remove(&common_rec_index).unwrap();
        let b = b_map.remove(&common_rec_index).unwrap();

        let (only_in_a, only_in_b) = compare_anchor_alignment_set(&a, &b);

        if !only_in_a.is_empty() {
            is_same = false;
            warn!("Only in 1st - {} - {:?}", common_rec_index, only_in_a);
        }
        if !only_in_b.is_empty() {
            is_same = false;
            warn!("Only in 2nd - {} - {:?}", common_rec_index, only_in_b);
        }
    }

    (is_same, a_map, b_map)
}

fn compare_anchor_alignment_set(
    a: &AHashSet<AnchorAlignmentResult>,
    b: &AHashSet<AnchorAlignmentResult>,
) -> (Vec<AnchorAlignmentResult>, Vec<AnchorAlignmentResult>) {
    let mut only_in_a: Vec<AnchorAlignmentResult> = a.iter().map(|v| v.clone()).collect();
    let mut only_in_b = Vec::new();

    for b_aar in b {
        let mut matched_index_in_a = None;
        for (a_idx, a_aar) in only_in_a.iter().enumerate() {
            if (
                a_aar.length == b_aar.length
            ) && (
                a_aar.penalty == b_aar.penalty
            ) &&(
                a_aar.position == b_aar.position
            ) {
                matched_index_in_a = Some(a_idx);
                break;
            }
        }

        match matched_index_in_a {
            Some(a_idx) => {
                only_in_a.remove(a_idx);
            },
            None => {
                only_in_b.push(b_aar.clone());
            },
        }
    }
    (only_in_a, only_in_b)
}