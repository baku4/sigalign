use super::{
    DefaultAligner,
    SelfDescAligner,
};

impl std::fmt::Debug for DefaultAligner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("DefaultAligner");
        let mode = match self.inner {
            SelfDescAligner::Local(_) => "Local",
            SelfDescAligner::SemiGlobal(_) => "SemiGlobal",
        };
        debug_struct.field("mode", &mode);
        // TODO: More detailed
        debug_struct.finish()
    }
}

impl DefaultAligner {
    pub fn is_local_mode(&self) -> bool {
        matches!(self.inner, SelfDescAligner::Local(_))
    }
    pub fn get_pattern_size(&self) -> u32 {
        match &self.inner {
            SelfDescAligner::Local(v) => v.get_pattern_size(),
            SelfDescAligner::SemiGlobal(v) => v.get_pattern_size(),
        }
    }
}