use anyhow::{Context, Result};
use flate2::read::ZlibDecoder;
use std::io::Read;

pub mod font_manager;
pub mod image_converter;

/// Decodes a Base64URL-encoded + Zlib/Deflate-compressed string.
///
/// Attempts Zlib (RFC 1950) first, then falls back to raw Deflate (RFC 1951).
pub fn decode(encoded: &str) -> Result<String> {
    use base64::{
        alphabet,
        engine::{self, general_purpose},
        Engine as _,
    };

    // Use a permissive engine that handles both padded and unpadded Base64URL strings (common in Kroki)
    let engine = engine::GeneralPurpose::new(
        &alphabet::URL_SAFE,
        general_purpose::NO_PAD.with_decode_padding_mode(engine::DecodePaddingMode::Indifferent),
    );

    let decoded_bytes = engine
        .decode(encoded)
        .context("Base64URL decode failed — input is not valid Base64URL")?;

    // Try Zlib first (RFC 1950)
    let mut decoder = ZlibDecoder::new(&decoded_bytes[..]);
    let mut s = String::new();
    match decoder.read_to_string(&mut s) {
        Ok(_) => return Ok(s),
        Err(e) => {
            tracing::debug!("Zlib decode failed ({}), trying raw Deflate", e);
        }
    }

    // Fallback to raw Deflate (RFC 1951) — often used by pako
    let mut decoder = flate2::read::DeflateDecoder::new(&decoded_bytes[..]);
    let mut s = String::new();
    match decoder.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => {
            // Distinguish decompression failure from UTF-8 encoding issues
            if e.kind() == std::io::ErrorKind::InvalidData {
                anyhow::bail!(
                    "Decompression failed — input is not valid Zlib or Deflate compressed data: {}",
                    e
                )
            } else {
                anyhow::bail!("Decompressed data is not valid UTF-8 text: {}", e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_graphviz() {
        // "digraph G { Hello -> World }" encoded with Python's zlib.compress + base64.urlsafe_b64encode
        let encoded = "eJxLyUwvSizIUHBXqFbwSM3JyVfQtVMIzy_KSVGoBQCJQglG";
        let result = decode(encoded);
        assert!(result.is_ok(), "decode failed: {:?}", result.err());
        let decoded = result.unwrap();
        assert_eq!(decoded, "digraph G { Hello -> World }");
    }

    #[test]
    fn test_decode_invalid_base64() {
        let result = decode("not-valid-base64!!!");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Base64URL"),
            "Expected Base64 error, got: {}",
            err
        );
    }

    #[test]
    fn test_decode_not_compressed() {
        use base64::{engine::general_purpose, Engine as _};
        // Valid base64 but not compressed data
        let encoded = general_purpose::URL_SAFE.encode("hello world");
        let result = decode(&encoded);
        // This should fail because "hello world" is not zlib/deflate compressed
        assert!(result.is_err());
    }
}
