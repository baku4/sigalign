use super::{
    FastaResult,
    PyReadResult,
    QueryResult,
    PyTargetResult,
    PyOperation,
};

impl FastaResult {
    pub fn to_flat_results(&self) -> Vec<FlatReadResult> {
        let vec_len: usize = self.num_alignments();
        let mut flat_results = Vec::with_capacity(vec_len);

        self.0.iter().for_each(|read_result| {
            read_result.push_flat_results(&mut flat_results);
        });
        flat_results
    }
}
impl PyReadResult {
    pub fn to_flat_results(&self) -> Vec<FlatReadResult> {
        let vec_len: usize = self.num_alignments();
        let mut flat_results = Vec::with_capacity(vec_len);
        self.push_flat_results(&mut flat_results);
        flat_results
    }
    #[inline]
    fn push_flat_results(&self, flat_results: &mut Vec<FlatReadResult>) {
        self.result.0.iter().for_each(|target_result| {
            target_result.alignments.iter().for_each(|alignment| {
                let flat_result = (
                    self.read.clone(),
                    target_result.index,
                    target_result.label.clone(),
                    alignment.penalty,
                    alignment.length,
                    alignment.query_position.0,
                    alignment.query_position.1,
                    alignment.target_position.0,
                    alignment.target_position.1,
                    operations_to_string(&alignment.operations),
                );
                flat_results.push(flat_result);
            });
        });
    }
}
pub type FlatReadResult = (
    String, // read
    u32,    // index of target
    Option<String>, // label of target
    u32,    // penalty
    u32,    // length
    u32,    // query start index
    u32,    // query end index
    u32,    // target start index
    u32,    // target end index
    String, // operations
);

impl QueryResult {
    pub fn to_flat_results(&self) -> Vec<FlatTargetResult> {
        let vec_len: usize = self.num_alignments();
        let mut flat_results = Vec::with_capacity(vec_len);

        self.0.iter().for_each(|target_result| {
            target_result.push_flat_results(&mut flat_results);
        });
        flat_results
    }
}
impl PyTargetResult {
    pub fn to_flat_results(&self) -> Vec<FlatTargetResult> {
        let vec_len: usize = self.num_alignments();
        let mut flat_results = Vec::with_capacity(vec_len);
        self.push_flat_results(&mut flat_results);
        flat_results
    }
    #[inline]
    fn push_flat_results(&self, flat_results: &mut Vec<FlatTargetResult>) {
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
                operations_to_string(&alignment.operations),
            );
            flat_results.push(flat_result);
        });
    }
}
pub type FlatTargetResult = (
    u32,    // index of target
    Option<String>, // label of target
    u32,    // penalty
    u32,    // length
    u32,    // query start index
    u32,    // query end index
    u32,    // target start index
    u32,    // target end index
    String, // operations
);

fn operations_to_string(operations: &Vec<PyOperation>) -> String {
    let string_ops: Vec<String> = operations.iter().map(|op| {
        format!("{}{}", op.case, op.count)
    }).collect();
    string_ops.concat()
}
