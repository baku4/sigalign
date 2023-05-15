/*!
Fasta reader can read various type of FASTA formatted file
*/
use super::{Result, error_msg};
use crate::test_data_path::{
    get_lf_fa_path,
    get_crlf_fa_path,
    get_two_line_fa_path,
};

use sigalign::utils::FastaReader;

#[test]
fn fasta_reader() {
    let lf_fa = get_lf_fa_path();
    let crlf_fa = get_crlf_fa_path();
    let two_line = get_two_line_fa_path();

    let lf_reader = FastaReader::from_path(lf_fa).unwrap();
    let crlf_reader = FastaReader::from_path(crlf_fa).unwrap();
    let two_line_reader = FastaReader::from_path(two_line).unwrap();

    for ((v1, v2), v3) in lf_reader.zip(crlf_reader).zip(two_line_reader) {
        assert_eq!(v1, v2);
        assert_eq!(v1, v3);
    }
}
