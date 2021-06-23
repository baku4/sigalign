mod anchor;

use fm_index::converter::RangeConverter;
use fm_index::suffix_array::{SuffixOrderSampledArray, SuffixOrderSampler};
use fm_index::FMIndex;
type FmIndex = FMIndex<u8, RangeConverter<u8>, SuffixOrderSampledArray>;

type SeqeunceLength = u32;

const FM_SUFFIX_LEVEL: usize = 2;

struct Aligner {
    //
}

impl Aligner {
    fn new() {

    }
    fn perform(ref_seq: &[u8] , qry_seq: &[u8]) { // -> Option<(Vec<Operation>, usize)>
        let index = Reference::fmindex(&ref_seq);
    }
    fn perform_with_index<T: AsRef<[u8]>>(ref_seq: &Reference<T> , qry_seq: &[u8]) { // -> Option<(Vec<Operation>, usize)>
        
    }
}

struct Reference<T: AsRef<[u8]>>{
    sequence: T,
    index: FmIndex
}
impl<T: AsRef<[u8]>> Reference<T> {
    fn new(sequence: T) -> Self {
        let fm_index =  Self::fmindex(&sequence);
        Self {
            sequence: sequence,
            index: fm_index,
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
    RefClip(SeqeunceLength),
    QryClip(SeqeunceLength),
}

struct AlignmentBlock {
    operations: Vec<Operation>,
    penalty: usize,
}

struct EmpKmer {
    odd: usize,
    even: usize,
}

impl EmpKmer {
    fn new(mismatch_penalty: usize, gapopen_penalty: usize, gapext_penalty: usize) -> Self {
        let mo: usize;
        let me: usize;
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
        Self {
            odd: mo,
            even: me,
        }
    }
}