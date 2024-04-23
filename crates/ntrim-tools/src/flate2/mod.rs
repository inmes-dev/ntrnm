use flate2::read::DeflateDecoder;
use std::io::prelude::*;

pub fn decompress_deflate(encoded: &[u8]) -> Vec<u8> {
    let mut decoder = DeflateDecoder::new(encoded);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded).unwrap();
    decoded
}

pub fn compress_deflate(decoded: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(decoded).unwrap();
    encoder.finish().unwrap()
}