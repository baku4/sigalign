pub type Cigar = Vec<(Operation, u32)>;
// use `shrink_to_fit`

pub enum Operation {
    Match,
    Subst,
    Ins,
    Del,
    RefClip,
    QryClip,
}
