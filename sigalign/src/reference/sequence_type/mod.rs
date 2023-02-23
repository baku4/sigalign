#[derive(Debug)]
pub struct SequenceType([bool; 256]);

impl SequenceType {
    #[inline]
    pub fn new(sequence: &[u8]) -> Self {
        let mut table = [false; 256];
        for chr in sequence {
            table[*chr as usize] = true;
        }
        Self(table)
    }
    #[inline]
    pub fn validate_query(&self, query: &[u8]) -> bool {
        for chr in query {
            if !self.0[*chr as usize] {
                return false
            }
        }
        true
    }
    #[inline]
    pub fn valid_characters(&self) -> Vec<u8> {
        self.0.iter().enumerate().filter_map(|(idx, c)| {
            if *c { Some(idx as u8) } else { None }
        }).collect()
    }
}

mod features;