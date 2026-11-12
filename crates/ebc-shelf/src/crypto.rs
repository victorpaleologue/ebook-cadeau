//! Small cryptographic helpers: SHA-256, HMAC-SHA-256, hex, constant-time comparison,
//! and the two signature schemes of `docs/api.md`: request signatures (§4.1, the shelf
//! signs its calls to the core) and the signed parameters of a `partner_file` link
//! request (§4.5, the core signs its calls to the shelf, as a redemption redirect §4.3).
//!
//! Everything here is pure: time is passed in by the caller.

use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

/// Largest clock skew accepted on a timestamp from the future, in seconds (§4.1).
pub const MAX_FUTURE_SKEW_SECS: i64 = 300;
/// Oldest timestamp accepted on a §4.5 link request, in seconds (as §4.3).
pub const LINK_REQUEST_MAX_AGE_SECS: i64 = 900;

/// Why a signature was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SignatureError {
    /// A parameter is missing or not well-formed.
    #[error("signature or timestamp missing or malformed")]
    Malformed,
    /// The timestamp is outside the accepted window.
    #[error("timestamp outside the accepted window")]
    Stale,
    /// The signature does not match.
    #[error("signature mismatch")]
    Mismatch,
}

/// Lower-case hexadecimal.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(char::from(DIGITS[usize::from(b >> 4)]));
        out.push(char::from(DIGITS[usize::from(b & 0x0f)]));
    }
    out
}

/// Decodes hexadecimal (either case); `None` when the input is not hex.
#[must_use]
pub fn unhex(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let nibble = |c: u8| match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    };
    bytes.chunks_exact(2).map(|pair| Some((nibble(pair[0])? << 4) | nibble(pair[1])?)).collect()
}

/// SHA-256.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// SHA-256 as lower-case hex.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    hex(&sha256(data))
}

/// HMAC-SHA-256 (RFC 2104), keys of any length.
#[must_use]
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    // The block-sized key as RFC 2104 prepares it (hashed when longer than a block, then
    // zero-padded), which allows the infallible fixed-size constructor.
    let mut block = [0u8; 64];
    if key.len() > block.len() {
        block[..32].copy_from_slice(&sha256(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let mut mac = <Hmac<Sha256> as KeyInit>::new(&block.into());
    mac.update(message);
    mac.finalize().into_bytes().into()
}

/// Constant-time equality (the length is not hidden).
#[must_use]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// `X-EBC-Signature` of a request to the core (§4.1): `v1=` + hex HMAC-SHA-256 of
/// `"{timestamp}.{METHOD}.{path-with-query}.{hex sha256(body)}"`.
#[must_use]
pub fn request_signature(
    secret: &str,
    timestamp: i64,
    method: &str,
    path_and_query: &str,
    body: &[u8],
) -> String {
    let message = format!(
        "{timestamp}.{}.{path_and_query}.{}",
        method.to_ascii_uppercase(),
        sha256_hex(body)
    );
    format!("v1={}", hex(&hmac_sha256(secret.as_bytes(), message.as_bytes())))
}

/// `ebc_sig` of a §4.5 link request: hex HMAC-SHA-256 of `"{gift_id}.{order_ref}.{ts}"`.
#[must_use]
pub fn link_request_signature(secret: &str, gift_id: &str, order_ref: &str, ts: i64) -> String {
    hex(&hmac_sha256(secret.as_bytes(), format!("{gift_id}.{order_ref}.{ts}").as_bytes()))
}

/// Verifies the signed parameters of a §4.5 link request from the core.
///
/// The timestamp must be at most [`LINK_REQUEST_MAX_AGE_SECS`] old and at most
/// [`MAX_FUTURE_SKEW_SECS`] in the future; the signature is compared in constant time.
///
/// # Errors
/// See [`SignatureError`].
pub fn verify_link_request(
    secret: &str,
    gift_id: &str,
    order_ref: &str,
    ts: &str,
    signature: &str,
    now: i64,
) -> Result<(), SignatureError> {
    if gift_id.is_empty() || order_ref.is_empty() || gift_id.len() > 128 || order_ref.len() > 128 {
        return Err(SignatureError::Malformed);
    }
    if ts.is_empty() || ts.len() > 12 || !ts.bytes().all(|b| b.is_ascii_digit()) {
        return Err(SignatureError::Malformed);
    }
    let timestamp: i64 = ts.parse().map_err(|_| SignatureError::Malformed)?;
    let provided = unhex(signature.trim()).ok_or(SignatureError::Malformed)?;
    if provided.len() != 32 {
        return Err(SignatureError::Malformed);
    }
    let age = now - timestamp;
    if !(-MAX_FUTURE_SKEW_SECS..=LINK_REQUEST_MAX_AGE_SECS).contains(&age) {
        return Err(SignatureError::Stale);
    }
    let expected =
        hmac_sha256(secret.as_bytes(), format!("{gift_id}.{order_ref}.{timestamp}").as_bytes());
    if ct_eq(&expected, &provided) { Ok(()) } else { Err(SignatureError::Mismatch) }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        assert_eq!(
            hex(&hmac_sha256(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
        assert_eq!(unhex("00FFa1"), Some(vec![0, 255, 161]));
        assert_eq!(unhex("0"), None);
        assert_eq!(unhex("zz"), None);
    }

    #[test]
    fn request_signature_is_the_documented_string() {
        let sig = request_signature("s", 1_800_000_000, "post", "/api/partner/v1/gifts", b"{}");
        let expected = format!("1800000000.POST./api/partner/v1/gifts.{}", sha256_hex(b"{}"));
        assert_eq!(sig, format!("v1={}", hex(&hmac_sha256(b"s", expected.as_bytes()))));
    }

    #[test]
    fn link_requests_good_stale_forged_malformed() {
        let (secret, gift, order, ts) = ("k-0123456789abcdef", "g_1", "c_2", 2_000_000);
        let sig = link_request_signature(secret, gift, order, ts);
        let t = ts.to_string();
        let check =
            |g: &str, o: &str, t: &str, s: &str, now| verify_link_request(secret, g, o, t, s, now);
        assert_eq!(check(gift, order, &t, &sig, ts), Ok(()));
        assert_eq!(check(gift, order, &t, &sig.to_uppercase(), ts + 900), Ok(()));
        assert_eq!(check(gift, order, &t, &sig, ts + 901), Err(SignatureError::Stale));
        assert_eq!(check(gift, order, &t, &sig, ts - 301), Err(SignatureError::Stale));
        assert_eq!(check("g_2", order, &t, &sig, ts), Err(SignatureError::Mismatch));
        assert_eq!(check(gift, "c_3", &t, &sig, ts), Err(SignatureError::Mismatch));
        assert_eq!(
            verify_link_request("other", gift, order, &t, &sig, ts),
            Err(SignatureError::Mismatch)
        );
        for (g, o, t, s) in [
            ("", order, t.as_str(), sig.as_str()),
            (gift, "", &t, &sig),
            (gift, order, "+2000000", &sig),
            (gift, order, "12x", &sig),
            (gift, order, "9999999999999", &sig),
            (gift, order, &t, "zz"),
            (gift, order, &t, &sig[..62]),
            (gift, order, &t, ""),
        ] {
            assert_eq!(check(g, o, t, s, ts), Err(SignatureError::Malformed), "{g} {o} {t} {s}");
        }
        let long = "g".repeat(129);
        assert_eq!(check(&long, order, &t, &sig, ts), Err(SignatureError::Malformed));
    }
}
