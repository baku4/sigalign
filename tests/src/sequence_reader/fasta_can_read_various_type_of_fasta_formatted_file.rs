/*!
Fasta reader can read various type of FASTA formatted file
*/
use crate::common::{
    Result, error_msg,
    test_data::DataForRefBuild,
};

use sigalign_utils::sequence_reader::{
    fasta::FastaReader,
    SeqRecord, SeqRefRecord, IdRecord, IdRefRecord,
    decompress::{get_gzip_decoder, get_zlib_decoder},
};

#[test]
// Test if the gzip and zlib compressed FASTA file can be read
fn generate_same_record_regardless_of_compression() {
    let fa = DataForRefBuild::LF.get_data_path();
    let gzip_fa = DataForRefBuild::Gz.get_data_path();
    let zlib_fa = DataForRefBuild::Zlib.get_data_path();

    let mut fa_reader = FastaReader::from_path(fa).unwrap();
    let gzip_file = std::fs::File::open(gzip_fa).unwrap();
    let mut gzip_reader = FastaReader::new(get_gzip_decoder(gzip_file));
    let zlib_file = std::fs::File::open(zlib_fa).unwrap();
    let mut zlib_reader = FastaReader::new(get_zlib_decoder(zlib_file));

    let mut fa_buf = Vec::new();
    let mut gzip_buf = Vec::new();
    let mut zlib_buf = Vec::new();
    while let Some(mut fa_record) = fa_reader.next() {
        let mut gzip_record = gzip_reader.next().unwrap();
        let mut zlib_record = zlib_reader.next().unwrap();

        // Compare sequences
        fa_buf.clear();
        gzip_buf.clear();
        zlib_buf.clear();
        fa_record.extend_id_buf(&mut fa_buf);
        gzip_record.extend_id_buf(&mut gzip_buf);
        zlib_record.extend_id_buf(&mut zlib_buf);
        assert_eq!(fa_buf, gzip_buf);
        assert_eq!(fa_buf, zlib_buf);

        // Compare ids
        fa_buf.clear();
        gzip_buf.clear();
        zlib_buf.clear();
        fa_record.extend_seq_buf(&mut fa_buf);
        gzip_record.extend_seq_buf(&mut gzip_buf);
        zlib_record.extend_seq_buf(&mut zlib_buf);
    }

    assert!(gzip_reader.next().is_none());
    assert!(zlib_reader.next().is_none());
}

#[test]
// Fasta reader can read various type of FASTA formatted file
fn generate_same_record_regardless_of_line_ending() {
    let lf_fa = DataForRefBuild::LF.get_data_path();
    let crlf_fa = DataForRefBuild::CRLF.get_data_path();
    let two_line = DataForRefBuild::TwoLine.get_data_path();

    let mut lf_reader = FastaReader::from_path(lf_fa).unwrap();
    let mut crlf_reader = FastaReader::from_path(crlf_fa).unwrap();
    let mut two_line_reader = FastaReader::from_path(two_line).unwrap();

    let mut lf_buf = Vec::new();
    let mut crlf_buf = Vec::new();
    let mut two_line_buf = Vec::new();

    while let Some(mut lf_record) = lf_reader.next() {
        let mut crlf_record = crlf_reader.next().unwrap();
        let mut two_line_record = two_line_reader.next().unwrap();

        // Compare sequences
        lf_buf.clear();
        crlf_buf.clear();
        two_line_buf.clear();
        lf_record.extend_id_buf(&mut lf_buf);
        crlf_record.extend_id_buf(&mut crlf_buf);
        two_line_record.extend_id_buf(&mut two_line_buf);
        assert_eq!(lf_buf, crlf_buf);
        assert_eq!(lf_buf, two_line_buf);
        
        // Compare ids
        lf_buf.clear();
        crlf_buf.clear();
        two_line_buf.clear();
        lf_record.extend_seq_buf(&mut lf_buf);
        crlf_record.extend_seq_buf(&mut crlf_buf);
        two_line_record.extend_seq_buf(&mut two_line_buf);
        assert_eq!(lf_buf, crlf_buf);
        assert_eq!(lf_buf, two_line_buf);
    }

    assert!(crlf_reader.next().is_none());
    assert!(two_line_reader.next().is_none());
}
