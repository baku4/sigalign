use super::{
    PyFastaAlignment, PyQueryAlignment, PyReadAlignment, PyTargetAlignment,
    PyAlignmentOperations, PyAlignmentOperation,
};

pub type FlatTargetAlignment = (
    u32,            // index of target
    Option<String>, // label of target
    u32,            // penalty
    u32,            // length
    u32,            // query start index
    u32,            // query end index
    u32,            // target start index
    u32,            // target end index
    String,         // operations
);

pub type FlatReadAlignment = (
    String,         // read
    bool,           // is forward
    u32,            // index of target
    Option<String>, // label of target
    u32,            // penalty
    u32,            // length
    u32,            // query start index
    u32,            // query end index
    u32,            // target start index
    u32,            // target end index
    String,         // operations
);

impl PyFastaAlignment {
    pub fn to_flat_results(&self) -> Vec<FlatReadAlignment> {
        let vec_len: usize = self.num_alignments();
        let mut flat_results = Vec::with_capacity(vec_len);

        self.0.iter().for_each(|read_result| {
            read_result.push_flat_results(&mut flat_results);
        });
        flat_results
    }
}
impl PyReadAlignment {
    pub fn to_flat_results(&self) -> Vec<FlatReadAlignment> {
        let vec_len: usize = self.num_alignments();
        let mut flat_results = Vec::with_capacity(vec_len);
        self.push_flat_results(&mut flat_results);
        flat_results
    }
    #[inline]
    fn push_flat_results(&self, flat_results: &mut Vec<FlatReadAlignment>) {
        self.result.0.iter().for_each(|target_result| {
            target_result.alignments.iter().for_each(|alignment| {
                let flat_read_result = (
                    self.read.clone(),
                    self.is_forward,
                    target_result.index,
                    target_result.label.clone(),
                    alignment.penalty,
                    alignment.length,
                    alignment.query_position.0,
                    alignment.query_position.1,
                    alignment.target_position.0,
                    alignment.target_position.1,
                    operations_to_cigars(&alignment.operations),
                );
                flat_results.push(flat_read_result);
            });
        });
    }
}

impl PyQueryAlignment {
    pub fn to_flat_results(&self) -> Vec<FlatTargetAlignment> {
        let vec_len: usize = self.num_alignments();
        let mut flat_results = Vec::with_capacity(vec_len);

        self.0.iter().for_each(|target_result| {
            target_result.push_flat_results(&mut flat_results);
        });
        flat_results
    }
}
impl PyTargetAlignment {
    pub fn to_flat_results(&self) -> Vec<FlatTargetAlignment> {
        let vec_len: usize = self.num_alignments();
        let mut flat_results = Vec::with_capacity(vec_len);
        self.push_flat_results(&mut flat_results);
        flat_results
    }
    #[inline]
    fn push_flat_results(&self, flat_results: &mut Vec<FlatTargetAlignment>) {
        self.alignments.iter().for_each(|alignment| {
            let flat_result = (
                self.index,
                self.label.clone(),
                alignment.penalty,
                alignment.length,
                alignment.query_position.0,
                alignment.query_position.1,
                alignment.target_position.0,
                alignment.target_position.1,
                operations_to_cigars(&alignment.operations),
            );
            flat_results.push(flat_result);
        });
    }
}

fn operations_to_cigars(operations: &[PyAlignmentOperations]) -> String {
    let string_ops: Vec<String> = operations
        .iter()
        .map(|op| format!(
            "{}{}",
            op.count,
            operation_to_cigar(&op.operation),
        ))
        .collect();
    string_ops.concat()
}

#[inline(always)]
fn operation_to_cigar(op: &PyAlignmentOperation) -> char {
    match op {
        PyAlignmentOperation::Match => '=',
        PyAlignmentOperation::Insertion => 'I',
        PyAlignmentOperation::Deletion => 'D',
        PyAlignmentOperation::Subst => 'X',
    }
}
