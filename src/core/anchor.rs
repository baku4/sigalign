use super::MinPenaltyForPattern;

mod preset;

type Anchors = Vec<Anchor>;

struct Anchor {
    query_position: usize,
    record_position: usize,
    size: usize,
    left_alignment: Alignment,
    left_check_points: CheckPoints,
    right_alignment: Alignment,
    right_check_points: CheckPoints,
    dropped: bool,
}
enum Alignment {
    Estimated(Estimation),
    Exact(Extension),
}

struct Estimation {
    penalty: usize,
    length: usize,
}

impl Estimation {
    fn new(penalty: usize, length: usize) -> Self {
        Self {
            penalty,
            length,
        }
    }
}

struct Extension {

}

struct CheckPoints(Vec<usize>);

impl CheckPoints {
    fn empty() -> Self {
        Self(Vec::new())
    }
}