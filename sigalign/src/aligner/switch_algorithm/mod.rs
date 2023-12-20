use super::{
    Aligner,
    DynamicAligner,
};
use sigalign_core::aligner::{
    LocalAligner, LocalWithLimitAligner,
    SemiGlobalAligner, SemiGlobalWithLimitAligner,
};

impl Aligner {
    /// Set the limit of the number of alignments for each query.
    pub fn set_limit(&mut self, limit: Option<u32>) {
        self.dynamic_aligner.set_limit(limit);
    }
    /// Change the algorithm to semi-global.
    /// Returns `false` if the algorithm is already semi-global.
    pub fn change_to_semi_global(&mut self) -> bool {
        self.dynamic_aligner.change_to_semi_global()
    }
    /// Change the algorithm to local.
    /// Returns `false` if the algorithm is already local.
    pub fn change_to_local(&mut self) -> bool {
        self.dynamic_aligner.change_to_local()
    }
}
impl DynamicAligner {
    fn set_limit(&mut self, limit: Option<u32>) {
        match limit {
            Some(limit) => {
                match self {
                    Self::Local(v) => {
                        let mut new = LocalAligner::new(v.get_regulator().clone());
                        std::mem::swap(v, &mut new);
                        *self = Self::LocalWithLimit(new.switch_to_limited(limit));
                    },
                    
                    Self::SemiGlobal(v) => {
                        let mut new = SemiGlobalAligner::new(v.get_regulator().clone());
                        std::mem::swap(v, &mut new);
                        *self = Self::SemiGlobalWithLimit(new.switch_to_limited(limit));
                    },
                    Self::LocalWithLimit(v) => {
                        v.set_limit(limit);
                    },
                    Self::SemiGlobalWithLimit(v) => {
                        v.set_limit(limit);
                    },
                }
            },
            None => {
                match self {
                    Self::LocalWithLimit(v) => {
                        let mut new = LocalWithLimitAligner::new(v.get_regulator().clone(), v.get_limit());
                        std::mem::swap(v, &mut new);
                        *self = Self::Local(new.switch_to_unlimited());
                    },
                    Self::SemiGlobalWithLimit(v) => {
                        let mut new = SemiGlobalWithLimitAligner::new(v.get_regulator().clone(), v.get_limit());
                        std::mem::swap(v, &mut new);
                        *self = Self::SemiGlobal(new.switch_to_unlimited());
                    },
                    _ => {},
                }
            },
        }
    }
    fn change_to_semi_global(&mut self) -> bool {
        match self {
            Self::Local(v) => {
                let regulator = v.get_regulator().clone();
                *self = Self::SemiGlobal(SemiGlobalAligner::new(regulator));
                true
            },
            Self::LocalWithLimit(v) => {
                let regulator = v.get_regulator().clone();
                *self = Self::SemiGlobalWithLimit(SemiGlobalWithLimitAligner::new(regulator, v.get_limit()));
                true
            },
            _ => false,
        }
    }
    fn change_to_local(&mut self) -> bool {
        match self {
            Self::SemiGlobal(v) => {
                let regulator = v.get_regulator().clone();
                *self = Self::Local(LocalAligner::new(regulator));
                true
            },
            Self::SemiGlobalWithLimit(v) => {
                let regulator = v.get_regulator().clone();
                *self = Self::LocalWithLimit(LocalWithLimitAligner::new(regulator, v.get_limit()));
                true
            },
            _ => false,
        }
    }
}
