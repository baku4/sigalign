mod data;
mod classic_wfa;
mod dp_pairwise;

mod accuracy {
    use super::*;
    use crate::alignment::*;

    #[test]
    fn test_two_files() {
        // Alignment options
        let score_per_length: f64 = 0.1;
        let minimum_length: usize = 100;
        let mismatch_penalty: usize = 4;
        let gapopen_penalty: usize = 3;
        let gapext_penalty: usize = 2;
        let using_cached_wf: bool = true;
        let get_minimum_penalty: bool = true;
        // calculate kmer
        let kmer = dp_pairwise::get_kmer(score_per_length, minimum_length, mismatch_penalty, gapopen_penalty, gapext_penalty);
        println!("# kmer: {:?}",kmer);
        // get dropout pariwise aligner
        let dop_aligner = Aligner::new(score_per_length, minimum_length, mismatch_penalty, gapopen_penalty, gapext_penalty, using_cached_wf, get_minimum_penalty);
        // get fasta records
        let (mut ref_records, mut qry_records) = data::get_fasta_records();
        while let Some(Ok(ref_record)) = ref_records.next() {
            println!("# reference record id: {:?}", ref_record.id());
            while let Some(Ok(qry_record)) = qry_records.next() {
                println!("# query record id: {:?}", qry_record.id());
                // Using DOP
                match dop_aligner.perform_with_sequence(
                    ref_record.seq(), qry_record.seq()
                ) {
                    Some(res) => {
                        println!("Res DOP:\n{:?}", res);
                    },
                    None => {
                        println!("Dropped in DOP");
                    }
                }
                // Using Dynamic Programming
                let dp_res = dp_pairwise::alignment(
                    ref_record.seq(), qry_record.seq(), score_per_length, minimum_length, kmer, mismatch_penalty, gapopen_penalty, gapext_penalty
                );
                if dp_res.len() == 0 {
                    println!("Dropped in DP");
                } else {
                    println!("Res DP:\n{:?}", dp_res);
                }
            }
        }
    }
}

mod performance {

}