use super::{SemiGlobalAligner, SemiGlobalWithLimitAligner};

impl SemiGlobalAligner {
    pub fn to_limited(self, limit: u32) -> SemiGlobalWithLimitAligner {
        SemiGlobalWithLimitAligner {
            regulator: self.regulator,
            workspace: self.workspace,
            limit,
        }
    }
}

impl SemiGlobalWithLimitAligner {
    pub fn to_unlimited(self) -> SemiGlobalAligner {
        SemiGlobalAligner {
            regulator: self.regulator,
            workspace: self.workspace,
        }
    }
}
