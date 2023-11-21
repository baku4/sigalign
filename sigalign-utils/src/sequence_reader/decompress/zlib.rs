use std::io::prelude::*;
use flate2::read::ZlibDecoder;

pub fn get_zlib_decoder<R: Read>(reader: R) -> ZlibDecoder<R> {
    ZlibDecoder::new(reader)
}
