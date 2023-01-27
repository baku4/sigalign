use wasm_bindgen::prelude::*;
use anyhow::Error as AnyhowErr;

#[inline]
pub fn err_to_js_err(err: AnyhowErr) -> JsError {
    JsError::new(&format!("{}", err))
}


use rand::prelude::*;
use super::{Reference, Aligner};

const MIN_QUERY_LEN: usize = 100;
const MAX_QUERY_LEN: usize = 200;
const SUBST_PROB: f64 = 0.8;
const EARLY_DROP_PROB: f64 = 0.4;
#[wasm_bindgen]
pub fn get_sample_query(
    reference: &Reference,
    aligner: &Aligner,
) -> Result<String, JsError> {
    let px = aligner.px();
    let po = aligner.po();
    let pe = aligner.pe();
    let ml = aligner.ml();
    let mppl = aligner.mppl();
    let record_count = reference.record_count();
    let allowed_chr_list = reference.allowed_characters();

    let mut rng = rand::thread_rng();

    let mut rec_idx_list: Vec<usize> = (0..record_count).collect();
    rec_idx_list.shuffle(&mut rng);

    // Get record seq
    for rec_idx in rec_idx_list {
        let rec_seq = reference.get_record_sequence(rec_idx);
        let rec_len = rec_seq.len();
        
        if rec_len < ml {
            continue
        }

        // Get original query seq
        let (min_qry_len, max_qry_len) = if rec_len < MIN_QUERY_LEN {
            (rec_len, rec_len)
        } else if rec_len < MAX_QUERY_LEN {
            (MIN_QUERY_LEN, rec_len)
        } else { // MAX_QUERY_LEN < record_len
            (MIN_QUERY_LEN, MAX_QUERY_LEN)
        };
        let qry_len = rng.gen_range(min_qry_len..=max_qry_len);
        let start_index_of_qry_in_rec = rng.gen_range(0..=rec_len-qry_len);
        let mut qry_seq = rec_seq[
            start_index_of_qry_in_rec..start_index_of_qry_in_rec+qry_len
        ].to_vec();

        // Mutate query
        let max_pen = (mppl * qry_len as f32).floor() as usize;
        let mut curr_pen = 0;
        let mut subst_count = 0;
        let mut indel_count = 0;
        let mut next_mut_is_subst = true;

        loop {
            if next_mut_is_subst {
                if curr_pen + px <= max_pen {
                    curr_pen += px;
                    subst_count += 1;

                    let to_early_drop = rng.gen_bool(EARLY_DROP_PROB);
                    if to_early_drop {
                        break;
                    }
                    
                    next_mut_is_subst = rng.gen_bool(SUBST_PROB);
                } else {
                    break;
                }
            } else { // is INDEL
                if curr_pen + po + pe <= max_pen {
                    curr_pen += po + pe;
                    indel_count += 1;

                    let to_early_drop = rng.gen_bool(EARLY_DROP_PROB);
                    if to_early_drop {
                        break;
                    }
                    
                    next_mut_is_subst = rng.gen_bool(SUBST_PROB);
                } else {
                    // Give the opportunity to be SUBST
                    next_mut_is_subst = true;
                }
            }
        }

        let to_mut_count = subst_count + indel_count;
        // sorted by descending order
        let to_mut_index = {
            let mut qry_idx_list: Vec<usize> = (0..qry_len).collect();
            qry_idx_list.shuffle(&mut rng); 
            let mut vec = qry_idx_list[0..to_mut_count].to_vec();
            vec.sort();
            vec.reverse();
            vec
        };
        let is_subst_list = {
            let mut vec = Vec::with_capacity(to_mut_count);
            (0..subst_count).for_each(|_| {
                vec.push(true)
            });
            (0..indel_count).for_each(|_| {
                vec.push(false)
            });
            vec
        };
        for (idx, is_subst) in to_mut_index.into_iter().zip(is_subst_list) {
            if is_subst {
                let org_chr = qry_seq[idx];
                let new_chr = get_random_muted_chr(
                    org_chr,
                    &mut rng,
                    &allowed_chr_list,
                );
                qry_seq[idx] = new_chr;
            } else {
                let is_insertion = rng.gen_bool(0.5);
                if is_insertion {
                    let ins_chr = get_random_chr(&mut rng, &allowed_chr_list);
                    qry_seq.insert(idx, ins_chr);
                } else {
                    qry_seq.remove(idx);
                }
            }
        }

        let qry = String::from_utf8(qry_seq).unwrap();
        return Ok(qry)
    }

    Err(JsError::new("No target longer than minimum length"))
}

fn get_random_muted_chr(
    chr: u8,
    rng: &mut ThreadRng,
    allowed_chr_list: &Vec<u8>,
) -> u8 {
    let matched_idx = allowed_chr_list.iter().position(|&v| v == chr).unwrap();
    let mut chr_idx = rng.gen_range(0..allowed_chr_list.len()-1);

    if chr_idx >= matched_idx {
        chr_idx += 1;
    }

    allowed_chr_list[chr_idx]
}
fn get_random_chr(
    rng: &mut ThreadRng,
    allowed_chr_list: &Vec<u8>,
) -> u8 {
    let chr_idx = rng.gen_range(0..allowed_chr_list.len());
    allowed_chr_list[chr_idx]
}