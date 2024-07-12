use std::env;
use log::info;
use sigalign_tests::{
    common::{
        init_logger,
        random_regulator::gen_random_regulator_with_seed,
        test_data_path::DataForValidation,
    },
    result_validation_with_dynamic_programming_matrix::{DpmTestUnit, DpmTestMode},
};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut thread_num = None;
    let mut start_seed = None;
    let mut seed_count = None;
    let mut overwrite = None;
    let mut mode = None;

    for (i, arg) in args.iter().enumerate() {
        match i {
            1 => mode = Some(arg.clone()),
            2 => thread_num = Some(arg.parse().unwrap()),
            3 => start_seed = Some(arg.parse().unwrap()),
            4 => seed_count = Some(arg.parse().unwrap()),
            5 => overwrite = Some(arg.parse().unwrap()),
            _ => (),
        }
    }

    let mode = match mode {
        Some(mode) => {
            if mode.contains("local") {
                DpmTestMode::LocalWithOneMat
            } else if mode.contains("semi") {
                DpmTestMode::SemiGlobal
            } else {
                panic!("Invalid mode: {}", mode);
            }
        },
        None => DpmTestMode::LocalWithOneMat,
    };
    init_logger();
    let thread_num = thread_num.unwrap_or(4);
    let start_seed = start_seed.unwrap_or(0);
    let seed_count = seed_count.unwrap_or(2);
    let overwrite = overwrite.unwrap_or(false);
    info!("# Test Specifics:");
    info!(" - Mode: {:?}", mode);
    info!(" - Thread Num: {}", thread_num);
    info!(" - Start Seed: {}", start_seed);
    info!(" - Seed Count: {}", seed_count);
    info!(" - Overwrite: {}", overwrite);
    let test_dataset = DataForValidation::Default;
    info!(" - Test Dataset: {:?}", test_dataset);
    
    for seed in start_seed..start_seed+seed_count {
        let regulator = gen_random_regulator_with_seed(seed);
        info!("# Seed : {} -> Regulator: {:?}", seed, regulator);
        let test_unit = DpmTestUnit::new(
            mode.clone(),
            test_dataset.clone(),
            regulator.0,
            regulator.1,
            regulator.2,
            regulator.3,
            regulator.4,
        ).unwrap();
        info!("Test Unit: {:?}", test_unit);
        let res = test_unit.save_results_to_cache_with_multiple_threads(
            thread_num,
            overwrite,
        );
        if let Err(e) = res {
            info!("{:?}", e)
        }
    }
}
