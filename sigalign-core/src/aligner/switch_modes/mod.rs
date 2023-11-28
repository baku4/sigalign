use super::{
    LocalAligner,
    LocalWithLimitAligner,
    SemiGlobalAligner,
    SemiGlobalWithLimitAligner,
    AllocationStrategy,
};

impl<A: AllocationStrategy> LocalAligner<A> {
    pub fn switch_to_limited(self, limit: u32) -> LocalWithLimitAligner<A> {
        LocalWithLimitAligner {
            regulator: self.regulator,
            space_manager: self.space_manager,
            limit,
        }
    }
}
impl<A: AllocationStrategy> LocalWithLimitAligner<A> {
    pub fn switch_to_unlimited(self) -> LocalAligner<A> {
        LocalAligner {
            regulator: self.regulator,
            space_manager: self.space_manager,
        }
    }
}
impl<A: AllocationStrategy> SemiGlobalAligner<A> {
    pub fn switch_to_limited(self, limit: u32) -> SemiGlobalWithLimitAligner<A> {
        SemiGlobalWithLimitAligner {
            regulator: self.regulator,
            space_manager: self.space_manager,
            limit,
        }
    }
}
impl<A: AllocationStrategy> SemiGlobalWithLimitAligner<A> {
    pub fn switch_to_unlimited(self) -> SemiGlobalAligner<A> {
        SemiGlobalAligner {
            regulator: self.regulator,
            space_manager: self.space_manager,
        }
    }
}
