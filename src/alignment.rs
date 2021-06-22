struct Aligner {
    //
}

impl Aligner {
    fn new() {

    }
    fn perform<T: AsRef<[u8]>>(ref_seq: &T , qry_seq: &T) { // -> Option<(Vec<Operation>, usize)>
        
    }
}

// data structure
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::SuffixOrderSampledArray;
use fm_index::FMIndex;

type FmIndex = FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>;

struct Reference<T: AsRef<[u8]>>{
    sequence: T,
    fm_index: FmIndex
}

enum Operation {
    
}