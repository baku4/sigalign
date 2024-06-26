use std::cmp::Ordering;

use ahash::AHashSet;

use super::{
    QueryAlignment,
    TargetAlignment,
    Alignment,
    AlignmentOperation, AlignmentPosition,
};

impl QueryAlignment {
    pub fn deduplicate(&mut self) {
        let mut paths = AHashSet::new();
        
        self.0.iter_mut().for_each(|v| {
            v.deduplicate_with_paths_buffer(&mut paths);
        })
    }
}

impl TargetAlignment {
    pub fn deduplicate(&mut self) {
        let mut paths = AHashSet::new();
        self.deduplicate_with_paths_buffer(&mut paths);
    }
    fn deduplicate_with_paths_buffer(&mut self, paths: &mut AHashSet<(u32, u32)>) {
        paths.clear();

        self.alignments.sort_unstable_by(|a, b| {
            cmp_anchor_alignment_result(a, b)
        });

        let temporary_vec = std::mem::take(&mut self.alignments);
        // - Same as 
        // let temporary_vec = std::mem::replace(
        //     &mut self.alignments,
        //     Vec::new(),
        // );
        
        self.alignments = temporary_vec.into_iter().filter(|v| {
            let path = v.get_path();
            if paths.is_disjoint(&path) {
                paths.extend(path);
                true
            } else {
                false
            }
        }).collect();
    }
}

fn cmp_anchor_alignment_result(
    a: &Alignment,
    b: &Alignment,
) -> Ordering {
    b.position.get_query_length().cmp(&a.position.get_query_length())
        .then(a.position.query.0.cmp(&b.position.query.0))
}

impl AlignmentPosition {
    fn get_query_length(&self) -> u32 {
        self.query.1 - self.query.0
    }
}

impl Alignment {
    fn get_path(&self) -> AHashSet<(u32, u32)> {
        let (mut query_index, mut target_index) = {
            let query_index = self.position.query.0;
            let target_index = self.position.target.0;
            (query_index, target_index)
        };
        let mut paths = AHashSet::new();
        self.operations.iter().for_each(|operation| {
            match operation.operation {
                AlignmentOperation::Match | AlignmentOperation::Subst => {
                    for _ in 0..operation.count {
                        paths.insert((query_index, target_index));
                        query_index += 1;
                        target_index += 1;
                    }
                },
                AlignmentOperation::Deletion => {
                    target_index += operation.count;
                },
                AlignmentOperation::Insertion => {
                    query_index += operation.count;
                },
            }
        });
        paths
    }
}
