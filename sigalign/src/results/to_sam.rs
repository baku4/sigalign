use super::{
    AnchorAlignmentResult,
    AlignmentOperations,
    AlignmentOperation,
};

impl AnchorAlignmentResult {
    #[inline]
    pub fn to_sam_record(
        &self,
        qname: &str,
        rname: &str,
        is_forward: bool,
    ) -> String {
        let cigar: String = self.operations.iter()
            .map(|op| op.to_cigar())
            .collect();
        let flag: u8 = if is_forward { 0 } else { 16 };

        let record = format!(
            "{}\t{}\t{}\t{}\t255\t{}\t*\t0\t0\t*\t*",
            qname,
            flag,
            rname,
            self.position.target.0 + 1,
            cigar,
        );
        record
    }
}

impl AlignmentOperations {
    #[inline]
    fn to_cigar(&self) -> String {
        format!("{}{}", self.count, self.operation.to_cigar_code())
    }
}

impl AlignmentOperation {
    #[inline]
    fn to_cigar_code(&self) -> u8 {
        match self {
            AlignmentOperation::Match | AlignmentOperation::Subst => b'M',
            AlignmentOperation::Insertion => b'I',
            AlignmentOperation::Deletion => b'D',
        }
    }
}
