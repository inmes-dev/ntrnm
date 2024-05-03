use std::io::{Read, Write};
use std::io::prelude::*;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use tokio;

pub fn decompress_deflate(encoded: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(encoded);
    let mut decoded = Vec::new();
    decoder.read_to_end(&mut decoded).unwrap();
    decoded
}

pub fn compress_deflate(decoded: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(decoded).unwrap();
    encoder.finish().unwrap()
}