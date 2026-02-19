use anyhow::{Context, Result};
use base64::prelude::*;
use flate2::read::ZlibDecoder;
use std::io::Read;

pub fn decode(encoded: &str) -> Result<String> {
    let decoded_bytes = BASE64_URL_SAFE
        .decode(encoded)
        .context("Base64 decode failed")?;

    // Try Zlib first (RFC 1950)
    let mut decoder = ZlibDecoder::new(&decoded_bytes[..]);
    let mut s = String::new();
    if decoder.read_to_string(&mut s).is_ok() {
        return Ok(s);
    }

    // Fallback to Deflate (RFC 1951) - often used by pako/raw deflate
    let mut decoder = flate2::read::DeflateDecoder::new(&decoded_bytes[..]);
    let mut s = String::new();
    decoder
        .read_to_string(&mut s)
        .context("Deep decode failed (both Zlib and Deflate)")?;
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_debug() {
        // "digraph G { Hello -> World }" encoded with Python's zlib.compress + base64.urlsafe_b64encode
        let encoded = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG";
        // Try to decode. If it fails, print error.
        match decode(encoded) {
            Ok(s) => println!("Decoded: {}", s),
            Err(e) => {
                println!("Decode error: {:?}", e);
                // Also try DeflateDecoder here to debug
                let decoded_bytes = BASE64_URL_SAFE.decode(encoded).unwrap();
                let mut decoder = flate2::read::DeflateDecoder::new(&decoded_bytes[..]);
                let mut s = String::new();
                match decoder.read_to_string(&mut s) {
                    Ok(_) => println!("Deflate Decoded: {}", s),
                    Err(e) => println!("Deflate Error: {}", e),
                }
            }
        }
    }
}
