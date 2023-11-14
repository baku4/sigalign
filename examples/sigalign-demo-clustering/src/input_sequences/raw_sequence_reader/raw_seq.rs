use crate::core::Sequence;

pub struct RawSequenceIterator {
    next_index: usize,
    internal: Vec<Vec<u8>>,
}

impl Iterator for RawSequenceIterator {
    type Item = Sequence;

    fn next(&mut self) -> Option<Self::Item> {
        if self.internal.len() < self.next_index {
            None
        } else {
            let inner = self.internal[self.next_index].clone();
            let sequence = Sequence {
                index: self.next_index as u32,
                id: None,
                inner,
            };
            self.next_index += 1;
            Some(sequence)
        }
    }
}

impl RawSequenceIterator {
    pub fn new(sequences: Vec<Vec<u8>>) -> Self {
        Self {
            next_index: 0,
            internal: sequences,
        }
    }
    pub fn reset(&mut self) {
        self.next_index = 0;
    }
}

