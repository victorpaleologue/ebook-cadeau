//! Bounded, strict stream decoding.
//!
//! `lopdf` decodes Flate streams leniently: a corrupt or truncated zlib stream yields
//! whatever could be inflated, without an error. That is the right call for a viewer and the
//! wrong one for a preflight, which would then approve a page whose tail it never saw. Plain
//! `FlateDecode` streams (the overwhelming majority of content streams) are therefore
//! inflated here with full error checking; other filter chains go through `lopdf`.

use flate2::{Decompress, FlushDecompress, Status};
use lopdf::{DecompressError, Object, Stream};

/// Why a stream could not be decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DecodeError {
    /// The decoded data would exceed the limit.
    TooLarge,
    /// The stream is corrupt or uses an unsupported filter.
    Invalid(String),
}

/// Decodes `stream`, producing at most `limit` bytes.
pub(crate) fn decode(stream: &Stream, limit: usize) -> Result<Vec<u8>, DecodeError> {
    if !stream.dict.has(b"Filter") {
        return if stream.content.len() > limit {
            Err(DecodeError::TooLarge)
        } else {
            Ok(stream.content.clone())
        };
    }
    let filters =
        stream.filters().map_err(|_| DecodeError::Invalid("malformed Filter entry".to_string()))?;
    if filters == [b"FlateDecode"] && !has_predictor(stream) {
        return inflate_strict(&stream.content, limit);
    }
    stream.get_plain_content_with_limit(limit).map_err(|error| match error {
        lopdf::Error::Decompress(DecompressError::MemoryLimitExceeded { .. }) => {
            DecodeError::TooLarge
        }
        other => DecodeError::Invalid(describe(&other)),
    })
}

/// Whether the stream's `DecodeParms` asks for a predictor (which `lopdf` knows how to undo).
fn has_predictor(stream: &Stream) -> bool {
    let predictor = |params: &Object| {
        params
            .as_dict()
            .ok()
            .and_then(|dict| dict.get(b"Predictor").ok())
            .and_then(|p| p.as_i64().ok())
            .is_some_and(|p| p > 1)
    };
    match stream.dict.get(b"DecodeParms") {
        Ok(Object::Array(items)) => items.iter().any(predictor),
        Ok(params) => predictor(params),
        Err(_) => false,
    }
}

/// Inflates a zlib stream, failing on corruption, truncation and oversized output.
fn inflate_strict(input: &[u8], limit: usize) -> Result<Vec<u8>, DecodeError> {
    const CHUNK: usize = 64 * 1024;
    // Some producers write zero-length streams with a Filter entry; viewers accept them.
    if input.is_empty() {
        return Ok(Vec::new());
    }
    let mut inflater = Decompress::new(true);
    let mut output: Vec<u8> = Vec::with_capacity(CHUNK.min(limit.saturating_add(1)));
    loop {
        if output.len() > limit {
            return Err(DecodeError::TooLarge);
        }
        if output.len() == output.capacity() {
            // Never reserve more than one byte beyond the limit: that byte detects overflow.
            let room = limit.saturating_add(1).saturating_sub(output.len());
            output.reserve(CHUNK.min(room.max(1)));
        }
        let consumed = usize::try_from(inflater.total_in()).unwrap_or(usize::MAX);
        let remaining = input.get(consumed..).unwrap_or(&[]);
        let (before_in, before_out) = (inflater.total_in(), inflater.total_out());
        let status = inflater
            .decompress_vec(remaining, &mut output, FlushDecompress::None)
            .map_err(|_| DecodeError::Invalid("corrupt FlateDecode data".to_string()))?;
        match status {
            Status::StreamEnd => {
                return if output.len() > limit { Err(DecodeError::TooLarge) } else { Ok(output) };
            }
            Status::Ok | Status::BufError => {
                let stalled =
                    inflater.total_in() == before_in && inflater.total_out() == before_out;
                if stalled && output.len() < output.capacity() {
                    return Err(DecodeError::Invalid("truncated FlateDecode data".to_string()));
                }
            }
        }
    }
}

/// An error with its causes, on one line.
pub(crate) fn describe(error: &dyn std::error::Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();
    let mut depth = 0;
    while let Some(cause) = source {
        text.push_str(": ");
        text.push_str(&cause.to_string());
        source = cause.source();
        depth += 1;
        if depth > 8 {
            break;
        }
    }
    text
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::io::Write;

    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use lopdf::dictionary;

    use super::*;

    fn deflate(data: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    fn flate_stream(compressed: Vec<u8>) -> Stream {
        Stream::new(dictionary! { "Filter" => "FlateDecode" }, compressed)
    }

    #[test]
    fn round_trips_plain_and_flate_streams() {
        let text = b"q 1 0 0 1 0 0 cm 0 0 10 10 re f Q\n".repeat(5_000);
        let raw = Stream::new(dictionary! {}, text.clone());
        assert_eq!(decode(&raw, 1 << 20).unwrap(), text);
        let flate = flate_stream(deflate(&text));
        assert_eq!(decode(&flate, 1 << 20).unwrap(), text);
        // An empty zlib stream decodes to nothing.
        assert_eq!(decode(&flate_stream(deflate(b"")), 10).unwrap(), b"");
    }

    #[test]
    fn enforces_the_size_limit_without_inflating_everything() {
        // 32 MiB of zeros compress to a few kilobytes: a small decompression bomb.
        let bomb = flate_stream(deflate(&vec![0_u8; 32 << 20]));
        assert!(bomb.content.len() < 64 * 1024);
        assert_eq!(decode(&bomb, 1 << 20), Err(DecodeError::TooLarge));
        // Exactly at the limit is fine, one byte over is not.
        let data = vec![7_u8; 1000];
        let stream = flate_stream(deflate(&data));
        assert_eq!(decode(&stream, 1000).unwrap().len(), 1000);
        assert_eq!(decode(&stream, 999), Err(DecodeError::TooLarge));
        let raw = Stream::new(dictionary! {}, data);
        assert_eq!(decode(&raw, 999), Err(DecodeError::TooLarge));
        assert_eq!(decode(&raw, 0), Err(DecodeError::TooLarge));
    }

    #[test]
    fn corrupt_and_truncated_flate_data_is_an_error() {
        let text = b"0 0 0 1 k 0 0 100 100 re f\n".repeat(200);
        let good = deflate(&text);
        // Truncated at every eighth: never silently partial.
        for cut in [1, 2, good.len() / 8, good.len() / 2, good.len() - 5, good.len() - 1] {
            let result = decode(&flate_stream(good[..cut].to_vec()), 1 << 20);
            assert!(matches!(result, Err(DecodeError::Invalid(_))), "cut at {cut}: {result:?}");
        }
        // A flipped byte in the middle breaks the data or the checksum.
        let mut flipped = good.clone();
        let middle = flipped.len() / 2;
        flipped[middle] ^= 0x55;
        assert!(matches!(decode(&flate_stream(flipped), 1 << 20), Err(DecodeError::Invalid(_))));
        // Zero bytes of data are accepted as an empty stream, as viewers do.
        assert_eq!(decode(&flate_stream(Vec::new()), 10).unwrap(), b"");
        // Not zlib at all.
        assert!(matches!(
            decode(&flate_stream(b"this is not zlib".to_vec()), 1 << 20),
            Err(DecodeError::Invalid(_))
        ));
    }

    #[test]
    fn other_filters_go_through_lopdf() {
        let hex = Stream::new(dictionary! { "Filter" => "ASCIIHexDecode" }, b"71 20 51>".to_vec());
        assert_eq!(decode(&hex, 100).unwrap(), b"q Q");
        let unknown = Stream::new(dictionary! { "Filter" => "DCTDecode" }, vec![0xFF, 0xD8]);
        assert!(matches!(decode(&unknown, 100), Err(DecodeError::Invalid(_))));
        let malformed = Stream::new(dictionary! { "Filter" => 12 }, vec![1, 2, 3]);
        assert_eq!(
            decode(&malformed, 100),
            Err(DecodeError::Invalid("malformed Filter entry".into()))
        );
    }
}
