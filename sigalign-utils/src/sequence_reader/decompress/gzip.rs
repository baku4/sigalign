use std::io::prelude::*;
use flate2::read::GzDecoder;

pub fn get_gzip_decoder<R: Read>(reader: R) -> GzDecoder<R> {
    GzDecoder::new(reader)
}
