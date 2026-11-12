//! Small cryptographic helpers: SHA-256, HMAC-SHA-256, hex, PBKDF2, and the two signature
//! schemes of `docs/api.md` (§4.1 request signatures, §4.3 redemption redirect).
//!
//! Everything here is pure: time is passed in by the caller.

use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};

/// Maximum clock skew accepted on signed API requests and webhooks (§4.1), in seconds.
pub const REQUEST_MAX_SKEW_SECS: i64 = 300;
/// Maximum age accepted on a signed redemption redirect (§4.3), in seconds.
pub const REDIRECT_MAX_AGE_SECS: i64 = 900;

/// Why a signature was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum SignatureError {
    /// A required header or parameter is missing or not well-formed.
    #[error("signature or timestamp missing or malformed")]
    Malformed,
    /// The timestamp is outside the accepted window.
    #[error("timestamp outside the accepted window")]
    Stale,
    /// The signature does not match.
    #[error("signature mismatch")]
    Mismatch,
}

/// Lower-case hexadecimal encoding.
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

/// Decodes hexadecimal (either case). `None` when the input is not valid hex.
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

/// SHA-256 of `data`.
#[must_use]
pub fn sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

/// SHA-256 of `data` as lower-case hex.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    hex(&sha256(data))
}

/// HMAC-SHA-256 (RFC 2104). Accepts keys of any length.
#[must_use]
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    // Prepare the block-sized key exactly as RFC 2104 specifies (hash when longer than a
    // block, then zero-pad), which lets us use the infallible fixed-size constructor.
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

/// Constant-time equality of two byte strings (length is not hidden).
#[must_use]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// PBKDF2-HMAC-SHA-256 with a 32-byte output (one block). Demo-grade password hashing.
#[must_use]
pub fn pbkdf2_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut first = Vec::with_capacity(salt.len() + 4);
    first.extend_from_slice(salt);
    first.extend_from_slice(&1u32.to_be_bytes());
    let mut u = hmac_sha256(password, &first);
    let mut out = u;
    for _ in 1..iterations.max(1) {
        u = hmac_sha256(password, &u);
        for (o, x) in out.iter_mut().zip(u) {
            *o ^= x;
        }
    }
    out
}

/// The string signed for an API request or webhook (§4.1):
/// `"{timestamp}.{METHOD}.{path-with-query}.{hex sha256(body)}"`.
#[must_use]
pub fn request_signing_string(
    timestamp: i64,
    method: &str,
    path_and_query: &str,
    body: &[u8],
) -> String {
    format!("{timestamp}.{}.{path_and_query}.{}", method.to_ascii_uppercase(), sha256_hex(body))
}

/// The value of the `X-EBC-Signature` header for a request (§4.1): `v1=<hex>`.
#[must_use]
pub fn request_signature(
    secret: &str,
    timestamp: i64,
    method: &str,
    path_and_query: &str,
    body: &[u8],
) -> String {
    let message = request_signing_string(timestamp, method, path_and_query, body);
    format!("v1={}", hex(&hmac_sha256(secret.as_bytes(), message.as_bytes())))
}

/// Verifies the `X-EBC-Timestamp` / `X-EBC-Signature` pair of an incoming signed request
/// (used for webhooks, §4.4).
///
/// # Errors
/// [`SignatureError::Malformed`] when a header is absent or unparsable,
/// [`SignatureError::Stale`] when the timestamp is more than
/// [`REQUEST_MAX_SKEW_SECS`] away from `now`, [`SignatureError::Mismatch`] otherwise.
pub fn verify_request_signature(
    secret: &str,
    timestamp_header: Option<&str>,
    signature_header: Option<&str>,
    method: &str,
    path_and_query: &str,
    body: &[u8],
    now: i64,
) -> Result<(), SignatureError> {
    let timestamp = parse_timestamp(timestamp_header)?;
    let provided = signature_header
        .and_then(|s| s.trim().strip_prefix("v1="))
        .and_then(unhex)
        .ok_or(SignatureError::Malformed)?;
    if (now - timestamp).abs() > REQUEST_MAX_SKEW_SECS {
        return Err(SignatureError::Stale);
    }
    let message = request_signing_string(timestamp, method, path_and_query, body);
    let expected = hmac_sha256(secret.as_bytes(), message.as_bytes());
    if ct_eq(&expected, &provided) { Ok(()) } else { Err(SignatureError::Mismatch) }
}

/// The `ebc_sig` parameter of a redemption redirect (§4.3):
/// hex HMAC-SHA-256 of `"{gift_id}.{order_ref}.{ts}"`.
#[must_use]
pub fn redirect_signature(secret: &str, gift_id: &str, order_ref: &str, ts: i64) -> String {
    let message = format!("{gift_id}.{order_ref}.{ts}");
    hex(&hmac_sha256(secret.as_bytes(), message.as_bytes()))
}

/// Verifies the signed parameters of a redemption redirect (§4.3).
///
/// The timestamp must not be older than [`REDIRECT_MAX_AGE_SECS`]; a timestamp from the
/// future is tolerated up to [`REQUEST_MAX_SKEW_SECS`] (clock drift between the two
/// services).
///
/// # Errors
/// See [`SignatureError`].
pub fn verify_redirect(
    secret: &str,
    gift_id: &str,
    order_ref: &str,
    ts: &str,
    signature: &str,
    now: i64,
) -> Result<(), SignatureError> {
    let timestamp = parse_timestamp(Some(ts))?;
    let provided = unhex(signature.trim()).ok_or(SignatureError::Malformed)?;
    if gift_id.is_empty() || order_ref.is_empty() {
        return Err(SignatureError::Malformed);
    }
    let age = now - timestamp;
    if !(-REQUEST_MAX_SKEW_SECS..=REDIRECT_MAX_AGE_SECS).contains(&age) {
        return Err(SignatureError::Stale);
    }
    let message = format!("{gift_id}.{order_ref}.{timestamp}");
    let expected = hmac_sha256(secret.as_bytes(), message.as_bytes());
    if ct_eq(&expected, &provided) { Ok(()) } else { Err(SignatureError::Mismatch) }
}

/// Parses a decimal Unix timestamp; rejects signs, spaces and anything longer than 12 digits
/// so that the string that is signed is the canonical one.
fn parse_timestamp(value: Option<&str>) -> Result<i64, SignatureError> {
    let text = value.ok_or(SignatureError::Malformed)?;
    if text.is_empty() || text.len() > 12 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(SignatureError::Malformed);
    }
    text.parse::<i64>().map_err(|_| SignatureError::Malformed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip_and_rejects_garbage() {
        assert_eq!(hex(&[0x00, 0xff, 0x1a]), "00ff1a");
        assert_eq!(unhex("00FF1a"), Some(vec![0x00, 0xff, 0x1a]));
        assert_eq!(unhex("0"), None);
        assert_eq!(unhex("zz"), None);
        assert_eq!(unhex("é0"), None);
        assert_eq!(unhex(""), Some(vec![]));
    }

    #[test]
    fn sha256_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hmac_rfc4231_vectors() {
        // Test case 1.
        assert_eq!(
            hex(&hmac_sha256(&[0x0b; 20], b"Hi There")),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        // Test case 2 (short key).
        assert_eq!(
            hex(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
        // Test case 6 (key longer than the block size: must be hashed first).
        assert_eq!(
            hex(&hmac_sha256(
                &[0xaa; 131],
                b"Test Using Larger Than Block-Size Key - Hash Key First"
            )),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn pbkdf2_known_vectors() {
        // RFC 7914 §11 / common PBKDF2-HMAC-SHA256 vectors ("password", "salt").
        assert_eq!(
            hex(&pbkdf2_sha256(b"password", b"salt", 1)),
            "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b"
        );
        assert_eq!(
            hex(&pbkdf2_sha256(b"password", b"salt", 2)),
            "ae4d0c95af6b46d32d0adff928f06dd02a303f8ef3c251dfd6e2d85a95474c43"
        );
        assert_eq!(
            hex(&pbkdf2_sha256(b"password", b"salt", 4096)),
            "c5e478d59288c841aa530db6845c4c8d962893a001ce4e11a4963873aa98134a"
        );
    }

    #[test]
    fn ct_eq_behaves_like_eq() {
        assert!(ct_eq(b"abc", b"abc"));
        assert!(!ct_eq(b"abc", b"abd"));
        assert!(!ct_eq(b"abc", b"abcd"));
        assert!(ct_eq(b"", b""));
    }

    #[test]
    fn request_signature_matches_the_documented_string() {
        let body = br#"{"a":1}"#;
        let ts = 1_800_000_000;
        let signing = request_signing_string(ts, "post", "/api/partner/v1/gifts?x=1", body);
        assert_eq!(
            signing,
            format!("1800000000.POST./api/partner/v1/gifts?x=1.{}", sha256_hex(body))
        );
        let header = request_signature("s3cret", ts, "POST", "/api/partner/v1/gifts?x=1", body);
        assert_eq!(header, format!("v1={}", hex(&hmac_sha256(b"s3cret", signing.as_bytes()))));
        assert_eq!(header.len(), 3 + 64);
    }

    #[test]
    fn request_verification_good_bad_stale_malformed() {
        let (secret, path, body, ts) = ("k", "/webhooks/ebc", b"{}".as_slice(), 1_000_000);
        let sig = request_signature(secret, ts, "POST", path, body);
        let ts_s = ts.to_string();
        let check = |secret: &str, t: Option<&str>, s: Option<&str>, body: &[u8], now: i64| {
            verify_request_signature(secret, t, s, "POST", path, body, now)
        };
        assert_eq!(check(secret, Some(&ts_s), Some(&sig), body, ts), Ok(()));
        assert_eq!(check(secret, Some(&ts_s), Some(&sig), body, ts + 300), Ok(()));
        assert_eq!(check(secret, Some(&ts_s), Some(&sig), body, ts - 300), Ok(()));
        assert_eq!(
            check(secret, Some(&ts_s), Some(&sig), body, ts + 301),
            Err(SignatureError::Stale)
        );
        assert_eq!(
            check(secret, Some(&ts_s), Some(&sig), body, ts - 301),
            Err(SignatureError::Stale)
        );
        assert_eq!(
            check("other", Some(&ts_s), Some(&sig), body, ts),
            Err(SignatureError::Mismatch)
        );
        assert_eq!(
            check(secret, Some(&ts_s), Some(&sig), b"{ }", ts),
            Err(SignatureError::Mismatch)
        );
        assert_eq!(check(secret, None, Some(&sig), body, ts), Err(SignatureError::Malformed));
        assert_eq!(check(secret, Some(&ts_s), None, body, ts), Err(SignatureError::Malformed));
        assert_eq!(
            check(secret, Some(&ts_s), Some(&sig[3..]), body, ts),
            Err(SignatureError::Malformed),
            "the v1= prefix is mandatory"
        );
        assert_eq!(
            check(secret, Some("+1000000"), Some(&sig), body, ts),
            Err(SignatureError::Malformed)
        );
        assert_eq!(
            check(secret, Some("99999999999999999999"), Some(&sig), body, ts),
            Err(SignatureError::Malformed)
        );
        // A truncated signature is a mismatch, not a panic.
        assert_eq!(
            check(secret, Some(&ts_s), Some(&sig[..sig.len() - 2]), body, ts),
            Err(SignatureError::Mismatch)
        );
    }

    #[test]
    fn redirect_verification_good_bad_stale() {
        let secret = "partner-secret";
        let (gift, order, ts) = ("g_123", "GE-2026-000001", 2_000_000);
        let sig = redirect_signature(secret, gift, order, ts);
        let ts_s = ts.to_string();
        assert_eq!(verify_redirect(secret, gift, order, &ts_s, &sig, ts + 10), Ok(()));
        assert_eq!(verify_redirect(secret, gift, order, &ts_s, &sig, ts + 900), Ok(()));
        assert_eq!(
            verify_redirect(secret, gift, order, &ts_s, &sig, ts + 901),
            Err(SignatureError::Stale)
        );
        assert_eq!(
            verify_redirect(secret, gift, order, &ts_s, &sig, ts - 301),
            Err(SignatureError::Stale),
            "timestamps far in the future are refused"
        );
        assert_eq!(
            verify_redirect(secret, "g_124", order, &ts_s, &sig, ts),
            Err(SignatureError::Mismatch)
        );
        assert_eq!(
            verify_redirect(secret, gift, "GE-2026-000002", &ts_s, &sig, ts),
            Err(SignatureError::Mismatch)
        );
        assert_eq!(
            verify_redirect("wrong", gift, order, &ts_s, &sig, ts),
            Err(SignatureError::Mismatch)
        );
        assert_eq!(
            verify_redirect(secret, gift, order, &ts_s, "nothex", ts),
            Err(SignatureError::Malformed)
        );
        assert_eq!(
            verify_redirect(secret, gift, order, "12x", &sig, ts),
            Err(SignatureError::Malformed)
        );
        assert_eq!(
            verify_redirect(secret, "", order, &ts_s, &sig, ts),
            Err(SignatureError::Malformed)
        );
        // The separator cannot be abused to move a character between fields unnoticed:
        // ("g_1", "23.X") and ("g_1.23", "X") sign different… the same string, so the
        // store must also check that the pair belongs to one of its own orders. This
        // documents the property the activation handler relies on.
        assert_eq!(
            redirect_signature(secret, "g_1", "23.X", ts),
            redirect_signature(secret, "g_1.23", "X", ts)
        );
    }
}
