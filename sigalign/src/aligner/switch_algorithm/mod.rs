use super::{
    Aligner,
    DynamicAligner,
};
use sigalign_core::aligner::{
    local::{LocalAligner, LocalWithLimitAligner},
    semi_global::{SemiGlobalAligner, SemiGlobalWithLimitAligner},
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
    pub fn set_limit(&mut self, limit: Option<u32>) {
        match limit {
            Some(limit) => {
                match self {
                    Self::Local(v) => {
                        let mut new = LocalAligner::new(v.regulator().clone());
                        std::mem::swap(v, &mut new);
                        *self = Self::LocalWithLimit(new.to_limited(limit));
                    },
                    Self::SemiGlobal(v) => {
                        let mut new = SemiGlobalAligner::new(v.regulator().clone());
                        std::mem::swap(v, &mut new);
                        *self = Self::SemiGlobalWithLimit(new.to_limited(limit));
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
                        let mut new = LocalWithLimitAligner::new(v.regulator().clone(), v.limit());
                        std::mem::swap(v, &mut new);
                        *self = Self::Local(new.to_unlimited());
                    },
                    Self::SemiGlobalWithLimit(v) => {
                        let mut new = SemiGlobalWithLimitAligner::new(v.regulator().clone(), v.limit());
                        std::mem::swap(v, &mut new);
                        *self = Self::SemiGlobal(new.to_unlimited());
                    },
                    _ => {},
                }
            },
        }
    }
    // Change the algorithm to semi-global. Returns `false` if the algorithm is already semi-global.
    fn change_to_semi_global(&mut self) -> bool {
        match self {
            Self::Local(v) => {
                let regulator = v.regulator().clone();
                *self = Self::SemiGlobal(SemiGlobalAligner::new(regulator));
                true
            },
            Self::LocalWithLimit(v) => {
                let regulator = v.regulator().clone();
                *self = Self::SemiGlobalWithLimit(SemiGlobalWithLimitAligner::new(regulator, v.limit()));
                true
            },
            _ => false,
        }
    }
    // Change the algorithm to local. Returns `false` if the algorithm is already local.
    fn change_to_local(&mut self) -> bool {
        match self {
            Self::SemiGlobal(v) => {
                let regulator = v.regulator().clone();
                *self = Self::Local(LocalAligner::new(regulator));
                true
            },
            Self::SemiGlobalWithLimit(v) => {
                let regulator = v.regulator().clone();
                *self = Self::LocalWithLimit(LocalWithLimitAligner::new(regulator, v.limit()));
                true
            },
            _ => false,
        }
    }
}
