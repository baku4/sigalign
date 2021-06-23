struct Aligner {
    //
}

impl Aligner {
    fn new() {

    }
    fn perform(ref_seq: &[u8] , qry_seq: &[u8]) { // -> Option<(Vec<Operation>, usize)>
        let fm_index = Reference::fmindex(&ref_seq);
    }
    fn perform_with_fmindex<T: AsRef<[u8]>>(ref_seq: &Reference<T> , qry_seq: &[u8]) { // -> Option<(Vec<Operation>, usize)>
        
    }
}

// data structure
use fm_index::converter::RangeConverter;
use fm_index::suffix_array::{SuffixOrderSampledArray, SuffixOrderSampler};
use fm_index::FMIndex;

type FmIndex = FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>;

const FM_SUFFIX_LEVEL: usize = 2;

struct Reference<T: AsRef<[u8]>>{
    sequence: T,
    fm_index: FmIndex
}
impl<T: AsRef<[u8]>> Reference<T> {
    fn new(sequence: T) -> Self {
        let fm_index =  Self::fmindex(&sequence);
        Self {
            sequence,
            fm_index,
        }
    }
    fn fmindex(sequence: &T) -> FmIndex {
        let seq = sequence.as_ref().iter().cloned().collect();
        // TODO: change the input ASCII code range
        let converter = RangeConverter::new(b'A', b'T');
        let sampler = SuffixOrderSampler::new().level(FM_SUFFIX_LEVEL);
        FMIndex::new(seq, converter, sampler)
    }
}

enum Operation {
    Match,
    Subst,
    Ins,
    Del,
    RefClip(u32),
    QryClip(u32),
}