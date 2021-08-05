use bio::io::fasta::{self, *};
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;

const FASTA_DIR: &str = "./src/tests/fasta";
const FASTA_FILES: [&str; 2] = ["./src/tests/fasta/ERR209055.fa", "./src/tests/fasta/ERR209056.fa"];

pub fn ref_fasta_records() -> Records<File> {
    fasta::Reader::from_file(FASTA_FILES[0]).unwrap().records()
}
pub fn qry_fasta_records() -> Records<File> {
    fasta::Reader::from_file(FASTA_FILES[1]).unwrap().records()
}

pub fn get_fasta_records() -> (Records<File>, Records<File>) {
    let mut ref_records = fasta::Reader::from_file(FASTA_FILES[0]).unwrap().records();
    let mut qry_records = fasta::Reader::from_file(FASTA_FILES[1]).unwrap().records();
    (ref_records, qry_records)
}

pub fn get_connected_map() -> HashMap<String, String> {
    let text = 
    // "k59_2225,k59_3595
    // k59_2225,k59_14365
    // k59_10301,k59_8627
    // k59_12501,k59_1808
    // k59_15442,k59_5224
    // k59_4429,k59_2565
    // k59_741,k59_11999
    // k59_6621,k59_10621
    // k59_5898,k59_15690
    // k59_4429,k59_4275
    // k59_8832,k59_12721
    // k59_16911,k59_16449
    // k59_5897,k59_5580
    // k59_14715,k59_869
    // k59_1494,k59_2266
    // k59_3673,k59_2409
    // k59_40,k59_15149
    // k59_16911,k59_11902
    // k59_39,k59_12692
    // k59_14716,k59_12339
    // k59_16912,k59_2348
    // k59_4432,k59_988
    // k59_744,k59_6428
    // k59_5899,k59_12109
    // k59_13981,k59_3911
    // k59_12502,k59_16693
    // k59_3673,k59_6196
    // k59_3676,k59_371
    // k59_6623,k59_10417
    // k59_7368,k59_4605
    // k59_13229,k59_8718
    // k59_3679,k59_13822
    // k59_11038,k59_17117
    // k59_11039,k59_11024
    // k59_14718,k59_12771
    // k59_3680,k59_12863
    // k59_4433,k59_3410
    // k59_13229,k59_5657
    // k59_14720,k59_16430
    // k59_15451,k59_10683
    // k59_1498,k59_4843
    // k59_746,k59_6660
    // k59_11040,k59_9742
    // k59_12505,k59_14220
    // k59_15452,k59_10535
    // k59_12504,k59_2652
    // k59_15452,k59_9166
    // k59_7370,k59_9648
    // k59_7369,k59_1248
    // k59_5904,k59_1104
    // k59_3681,k59_9855
    // k59_13984,k59_9003
    // k59_8834,k59_1009
    // k59_747,k59_2849
    // k59_8086,k59_15590
    // k59_8085,k59_7028
    // k59_6628,k59_10335
    // k59_1502,k59_14333
    // k59_8834,k59_7795
    // k59_2229,k59_4090
    // k59_8088,k59_14810
    // k59_13985,k59_143
    // k59_5905,k59_6309
    // k59_12506,k59_14776
    // k59_12506,k59_6242
    // k59_2951,k59_2750
    // k59_15457,k59_11134
    // k59_8836,k59_5345
    // k59_8090,k59_9873
    // k59_13985,k59_11196
    // k59_2952,k59_4986
    // k59_49,k59_1278
    // k59_3685,k59_10219
    // k59_11044,k59_9501
    // k59_13986,k59_6734
    // k59_5907,k59_16796
    // k59_1503,k59_5921
    // k59_11045,k59_15304
    // k59_750,k59_16597
    // k59_12508,k59_4014
    // k59_12509,k59_7919
    // k59_751,k59_6479
    // k59_6632,k59_13140
    // k59_8839,k59_3318
    // k59_16178,k59_13297
    // k59_6633,k59_15274
    // k59_16178,k59_9101
    // k59_2234,k59_16677
    // k59_1506,k59_9690
    // k59_8095,k59_3934
    // k59_2956,k59_10959
    // k59_8840,k59_3490
    "k59_16179,k59_2292
    k59_5908,k59_4850
    k59_6635,k59_7700
    k59_13992,k59_5925
    k59_13233,k59_2698
    k59_13233,k59_12005
    k59_14721,k59_16992
    k59_13233,k59_7776
    k59_13233,k59_12675
    k59_13993,k59_8973
    k59_10302,k59_8002
    k59_12510,k59_4513
    k59_9547,k59_13963
    k59_8844,k59_4555
    k59_3689,k59_5956
    k59_53,k59_3211
    k59_6636,k59_2136
    k59_4442,k59_12329
    k59_7379,k59_14727
    k59_9547,k59_6299
    k59_3690,k59_12045
    k59_7379,k59_13476
    k59_7380,k59_7222
    k59_4443,k59_5382
    k59_8100,k59_2201
    k59_3691,k59_5183
    k59_7380,k59_14492
    k59_11051,k59_8880
    k59_13994,k59_3015
    k59_8847,k59_8371
    k59_6640,k59_11948
    k59_5913,k59_15935
    k59_13994,k59_6885
    k59_5140,k59_15919
    k59_4445,k59_562
    k59_9550,k59_11529
    k59_1510,k59_15567
    k59_55,k59_3399
    k59_5142,k59_12907
    k59_13996,k59_1461
    k59_14724,k59_9402
    k59_4446,k59_15273
    k59_6643,k59_6732
    k59_11052,k59_7872
    k59_7381,k59_3333
    k59_755,k59_8386
    k59_2960,k59_15131
    k59_2960,k59_13053
    k59_6644,k59_2196
    k59_3693,k59_14337
    k59_11052,k59_10496
    k59_57,k59_776
    k59_1511,k59_15347
    k59_5143,k59_7256
    k59_15458,k59_5674
    k59_57,k59_300
    k59_756,k59_15161
    k59_12516,k59_8558
    k59_8102,k59_7294
    k59_8853,k59_3475
    k59_1511,k59_11402
    k59_756,k59_13924
    k59_5144,k59_12178
    k59_8856,k59_2248
    k59_59,k59_12252
    k59_5915,k59_4821
    k59_14728,k59_14494
    k59_8102,k59_6325
    k59_5915,k59_3455
    k59_15461,k59_16580
    k59_1513,k59_7349
    k59_3695,k59_15489
    k59_10310,k59_16827
    k59_10310,k59_16002
    k59_1514,k59_1512";
    let rows: Vec<&str> = text.split("\n").collect();
    let data: Vec<Vec<String>> = rows.into_iter().map(|v| v.trim().split(",").map(|x| x.to_string()).collect()).collect();
    let map: HashMap<String, String> = data.into_iter().map(|vec| (vec[0].clone(), vec[1].clone())).collect();
    map
}

fn print_fasta_list() {
    let fasta_path = Path::new(FASTA_DIR);
    for entry in fasta_path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            println!("{:?}", entry.path());
        }
    }
}

pub fn get_test_data() -> Vec<(String, String)> {
    let data = vec![
        (
            // >ERR209056;k59_14625
            "ATAAACATAAATACATATGCGGGAGAGCTTCCGCTGACTCCCACGACCACATCCATCAGACGCTCCGGTACGACACTTGCGATGCCGAAGGAACGGAGCAGGGAAAGCACCTGCTCTGTCTCATCCTCCGTGACATTTTCTCCGATGCATACACCGGTACAGCCCTCCAGTACCAGGGCCGGCGTATTGGGCATGCAGCGGATCAGCTTAAGCTCCGGTCTGTCAAATGCATCCTTC".to_string(),
            // >ERR209055;k59_2911
            "TTCACCGGGATGTTTTCCGGTCTCCAGCACCATTTTGGCACTGCCGAGTACGGACTGTGCTGCGAATTCATAGGCCTGTTTTCTGGGCATTCCGGCTGCCACAGCTTCATCCGCCATGGCCTCAATAAACATAAATACATATGCGGGAGAGCTTCCGCTGACTCCCACGACCACATCCATCAGACGCTCCGGTACGACACTTGCGATGCCGAAGGAACGGAGCAGGGAAAGCACCTGCTCTGTCTCATCCTCCGTGACATTTTC".to_string()
        ),
        (
            "ATAAACATAAATACATATGCGGGAGAGCTTCCGCTGACTCCCACGACCACATCCATCAGACGCTCCGGTACGACACTTGCGATGCCGAAGGAACGGAGCAGGGAAAGCACCTGCTCTGTCTCATCCTCCGTGACATTTTCTCCGATGCATACACCGGTACAGCCCTCCAGTACCAGGGCCGGCGTATTGGGCATGCAGCGGATCAGCTTAAGCTCCGGTCTGTCAAATGCATCCTTC".to_string(),
            "CCGCTGACTCCCACGACCACATCCATCAGACGCTCCGGTACGACACTTGCGATGCCGAAGGACGGAGCAGGGAAAGCACCTGCTCTGTCTCATCCTCCGTGACATTTTCTCCGATGCATACACCGGTACAGCCCTCCAGTACCAGGGCCGGCGTATTGGGCATGCAGCGGATCAGCTTAAGCTCCGGTCTGTCAAATTCATCCTTCGACAAACCTGTGGCCAAAACTCTTGAAGTCAGTAAATATCCTGCCATTTCTTGCTGCTTCTCATATCGCGACTGAATATCAATGCCAGATGACAGGGCAGTAATTGATTGAAGT".to_string(),
        )
    ];
    data
}