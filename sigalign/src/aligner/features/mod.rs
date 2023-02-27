// mod debug;
// use super::AlignmentRegulator;

// impl std::fmt::Debug for AlignmentRegulator {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Regulator")
//             .field("penalties", &self.penalties)
//             .field("cutoffs", &self.cutoff)
//             .field("gap_extend_penalty", &self.get_gap_extend_penalty())
//             .field("mimy", &self.y)
//             .finish() 
//     }
// }