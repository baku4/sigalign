use super::{
    InMemoryStorage,
    LabelStorage,
};

// Label Storage
impl LabelStorage for InMemoryStorage {
    fn label_of_target_unchecked(&self, target_index: u32) -> String {
        unsafe {
            String::from_utf8_unchecked(
                self.concatenated_label.as_bytes()[
                    self.label_index[target_index as usize]
                    ..self.label_index[target_index as usize +1]
                ].to_vec()
            )
        }
    }
}