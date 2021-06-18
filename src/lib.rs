mod classic_wfa;

#[derive(Debug)]
struct AlignmentConfig {
    // Gap-affine penalties
    mismatch_penalty: u8,
    gapopen_penalty: u8,
    gapext_penalty: u8,
    // Cutoff value
    minimum_length: u32,
    cutoff: f64,
    // EMP for odd/even kmer block
    odd_emp: u8,
    even_emp: u8,
    // Kmer-size
    kmer_size: u16,
}

impl AlignmentConfig {
    fn new(mismatch_penalty: u8, gapopen_penalty: u8, gapext_penalty: u8, minimum_length: u32, cutoff: f64) -> Self {
        let (odd_emp, even_emp) = Self::kmer_emp(mismatch_penalty, gapopen_penalty, gapext_penalty);
        let kmer_size = Self::kmer_calculation(minimum_length, cutoff, odd_emp, even_emp);
        Self {
            mismatch_penalty,
            gapopen_penalty,
            gapext_penalty,
            minimum_length,
            cutoff,
            odd_emp,
            even_emp,
            kmer_size,
        }
    }
    fn kmer_emp(mismatch_penalty: u8, gapopen_penalty: u8, gapext_penalty: u8) -> (u8, u8){
        let mo: u8;
        let me: u8;
        if mismatch_penalty <= gapopen_penalty + gapext_penalty {
            mo = mismatch_penalty;
            if mismatch_penalty * 2 <= gapopen_penalty + (gapext_penalty * 2) {
                me = mismatch_penalty;
            } else {
                me = gapopen_penalty + (gapext_penalty * 2) - mismatch_penalty;
            }
        } else {
            mo = gapopen_penalty + gapext_penalty;
            me = gapext_penalty;
        }
        (mo, me)
    }
    fn kmer_calculation(minimum_length: u32, cutoff: f64, odd_emp: u8, even_emp: u8) -> u16 {
        let mut i: u8 = 1;
        let mut kmer_size: f64;
        loop {
            kmer_size = (((minimum_length+2) as f64/(2*i) as f64) - 1_f64).ceil();
            if (i*(odd_emp + even_emp)) as f64 > cutoff * 2_f64 * (((i+1) as f64)*kmer_size-1_f64) {
                break;
            } else {
                i += 1;
            }
        }
        kmer_size as u16
    }
}

mod test {
    use super::*;
    #[test]
    fn config_test() {
        let mismatch_penalty:u8 = 3;
        let gapopen_penalty: u8 = 4;
        let gapext_penalty: u8 = 2;
        let minimum_length: u32 = 100;
        let cutoff: f64 = 0.05;

        let config = AlignmentConfig::new(
            mismatch_penalty,
            gapopen_penalty,
            gapext_penalty,
            minimum_length,
            cutoff
        );

        println!("{:?}", config);
    }
}