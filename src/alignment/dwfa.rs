type WF_SCORE = usize;
type WF_K = i32;
type FR_POINT = u32;

type BACTRACE_M = u8;
type BACTRACE_I = u8;
type BACTRACE_D = u8;

#[derive(Clone)]
struct WFS {
    m_fr: FR_POINT,
    i_fr: FR_POINT,
    d_fr: FR_POINT,
    m_bt: BACTRACE_M,
    i_bt: BACTRACE_I,
    d_bt: BACTRACE_D,
}

impl Default for WFS {
    fn default() -> Self {
        Self {
            ..Default::default()
        }
    }
}

type WaveFront = Vec<WFS>;

// TODO: vector from raw
fn empty_wf(size: usize) -> WaveFront {
    unsafe {
        vec![std::mem::zeroed(); size]
    }
}

struct Penalties {
    // penalties
    x: usize,
    o: usize,
    e: usize,
    // quot & rem
    xpe_q: usize,
    xpe_r: usize,
    ope_q: usize,
    ope_r: usize,
}

impl Penalties {
    #[inline]
    fn get_max_k(&self, score: WF_SCORE) -> usize {
        if score < self.o {
            1
        } else {
            (score - self.o) / self.e
        }
    }
    #[inline]
    fn get_index(&self, score: WF_SCORE, k: WF_K) -> usize {
        if score < self.o {
            score
        } else {
            let s_m_po = score - self.o;
            let q = s_m_po / self.e;
            let r = s_m_po % self.e;
            self.o + (q*q + q - 1)*self.e + (2*q-1)*r + q + k as usize
        }
    }
    #[inline]
    fn get_wf_size(&self, panalty_spare: WF_SCORE) -> usize {
        if panalty_spare < self.o {
            panalty_spare + 1
        } else {
            let s_m_po = panalty_spare - self.o;
            let q = s_m_po / self.e;
            let r = s_m_po % self.e;
            self.o + (q*q + q - 1)*self.e + (2*q-1)*r + 2*q + 1
        }
    }
}

fn dropout_wf_align(
    qry_seq: &[u8], ref_seq: &[u8],
    penalty_spare: usize, spl: f64,
    penalties: &Penalties,
) {
    // penalties: [x, o, e]
    let n = qry_seq.len();
    let m = ref_seq.len();

    // init
    let wf_size = penalties.get_wf_size(penalty_spare);
    let mut wf: WaveFront = empty_wf(wf_size);

    let mut score: usize = 0;
    let mut wf = {
        // let m_component = Component::new_with_first(0, 0);
        // let wf_score: WFscore = [Some(m_component), None, None];
        // vec![Some(wf_score)]
    };
}
