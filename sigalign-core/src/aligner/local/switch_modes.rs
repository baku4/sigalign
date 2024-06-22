use super::{LocalAligner, LocalWithLimitAligner};

impl LocalAligner {
    pub fn to_limited(self, limit: u32) -> LocalWithLimitAligner {
        LocalWithLimitAligner {
            regulator: self.regulator,
            workspace: self.workspace,
            limit,
        }
    }
}

impl LocalWithLimitAligner {
    pub fn to_unlimited(self) -> LocalAligner {
        LocalAligner {
            regulator: self.regulator,
            workspace: self.workspace,
        }
    }
}
