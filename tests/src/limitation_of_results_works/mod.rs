// Tests limited version of SemiGlobal and Local algorithms

use std::fs::File;

use crate::common::{
    init_logger,
    test_data::DataForValidation,
    random_regulator::gen_random_regulator,
};
use ahash::AHashSet as HashSet;
use log::info;
use sigalign::{
    algorithms::{Algorithm, Local, LocalWithLimit, SemiGlobal, SemiGlobalWithLimit}, results::{Alignment, QueryAlignment}, Aligner, Reference, ReferenceBuilder
};
use sigalign_utils::sequence_reader::{fasta::FastaReader, SeqRecord as _};

const SEED_START: u64 = 0;
const SEED_COUNT: u64 = 5;
const MAX_SUBST_PERCENT: u32 = 2;
const TEST_LIMITS: [u32; 5] = [1000, 100, 10, 1, 0];

#[test]
fn test_limited_semi_global_works() {
    let default_aligner_generator = |px, po, pe, minl, maxp| {
        Aligner::new(SemiGlobal::new(
            px, po, pe, minl, maxp
        ).unwrap()
    )};

    let limited_aligner_generator = |px, po, pe, minl, maxp, limit| {
        Aligner::new(SemiGlobalWithLimit::new(
            px, po, pe, minl, maxp, limit
        ).unwrap(),
    )};

    test_limit_works(&default_aligner_generator, &limited_aligner_generator);
}

#[test]
fn test_limited_local_works() {
    let default_aligner_generator = |px, po, pe, minl, maxp| {
        Aligner::new(Local::new(
            px, po, pe, minl, maxp
        ).unwrap()
    )};

    let limited_aligner_generator = |px, po, pe, minl, maxp, limit| {
        Aligner::new(LocalWithLimit::new(
            px, po, pe, minl, maxp, limit
        ).unwrap(),
    )};

    test_limit_works(&default_aligner_generator, &limited_aligner_generator);
}

fn test_limit_works<A1, A2, F1, F2>(
    default_aligner_generator: &F1,
    limited_aligner_generator: &F2,
) where
    A1: Algorithm,
    A2: Algorithm,
    F1: Fn(u32, u32, u32, u32, f32) -> Aligner<A1>,
    F2: Fn(u32, u32, u32, u32, f32, u32) -> Aligner<A2>,
{
    init_logger();

    let test_data = DataForValidation::Default;
    info!("Test data: {:?}", test_data);
    let (ref_file, qry_file) = test_data.get_data_paths();

    let reference = ReferenceBuilder::new().add_fasta_file(&ref_file).unwrap().build().unwrap();

    for seed in SEED_START..SEED_START + SEED_COUNT {
        let regulator = gen_random_regulator(MAX_SUBST_PERCENT, seed);
        info!("Start to compare with regulators: {:?} (seed: {})", regulator, seed);

        let mut default_aligner = default_aligner_generator(
            regulator.0,
            regulator.1,
            regulator.2,
            regulator.3,
            regulator.4,
        );

        let mut limited_aligners = Vec::new();
        for limit in TEST_LIMITS.iter() {
            let limited_aligner = limited_aligner_generator(
                regulator.0,
                regulator.1,
                regulator.2,
                regulator.3,
                regulator.4,
                *limit,
            );
            limited_aligners.push(limited_aligner);
        }

        let mut fasta_reader = FastaReader::new(
            std::fs::File::open(&qry_file).unwrap()
        );
        let mut query_buffer = Vec::new();
        let mut query_index = 0;
        while let Some(mut record) = fasta_reader.next() {
            query_buffer.clear();
            if query_index % 1000 == 0 {
                info!("Processed {} queries", query_index);
            }
            record.extend_seq_buf(&mut query_buffer);

            let default_results = default_aligner.align(&query_buffer, &reference);
            let default_results_set = query_alignment_to_set(default_results);

            for (limited_aligner, &limit) in limited_aligners.iter_mut()
                .zip(TEST_LIMITS.iter())
            {
                let limited_results = limited_aligner.align(&query_buffer, &reference);

                // Check the count of alignments
                assert!(limited_results.count_alignments() <= limit as usize);

                // Check if the results are contained in the default results
                let limited_results_set = query_alignment_to_set(limited_results);
                let default_is_superset = default_results_set.is_superset(&limited_results_set);
                if !default_is_superset {
                    info!("Default results:");
                    default_results_set.iter().for_each(|x| info!("{:?}", x));
                    info!("Limited results:");
                    limited_results_set.iter().for_each(|x| info!("{:?}", x));
                    panic!("Limited results are not contained in the default results");
                }
            }

            query_index += 1;
        }
    }
}

fn query_alignment_to_set(query_alignment: QueryAlignment) -> HashSet<(u32, Alignment)> {
    query_alignment.0.into_iter()
        .map(|x| {
            x.alignments.into_iter().map(move |y| (x.index, y) )
        })
        .flatten()
        .collect::<HashSet<_>>()
}
