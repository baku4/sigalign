#[derive(Debug)]
pub struct SequenceType([bool; 256]);

impl SequenceType {
    #[inline]
    pub fn infer_from_sequence(sequence: &[u8]) -> Self {
        let mut table = [false; 256];
        for chr in sequence {
            table[*chr as usize] = true;
        }
        Self(table)
    }
    #[inline]
    pub fn is_alignable(&self, query: &[u8]) -> bool {
        !query.iter().any(|chr| !self.0[*chr as usize])
    }
    #[inline]
    pub fn alignable_sequence(&self) -> Vec<u8> {
        self.0.iter().enumerate().filter_map(|(idx, c)| {
            if *c { Some(idx as u8) } else { None }
        }).collect()
    }
}

mod extensions;