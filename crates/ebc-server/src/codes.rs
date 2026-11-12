//! Secrets the server mints and how they are stored: gift codes and pair codes (kept only
//! as HMAC-SHA-256 under the server secret), bearer tokens (kept as SHA-256), random
//! identifiers, and the signatures of the partner protocol (`docs/api.md` §4).

use ebc_model::code::{GiftCode, PairCode};
use hmac::{Hmac, KeyInit, Mac};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::clock::{Entropy, EntropyError};

type HmacSha256 = Hmac<Sha256>;

/// Lower-case hexadecimal encoding.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(DIGITS[usize::from(byte >> 4)]));
        out.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    out
}

/// Decodes hexadecimal (either case); `None` for odd lengths or other characters.
#[must_use]
pub fn unhex(text: &str) -> Option<Vec<u8>> {
    let bytes = text.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }
    let nibble = |b: u8| match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    };
    bytes.chunks_exact(2).map(|pair| Some((nibble(pair[0])? << 4) | nibble(pair[1])?)).collect()
}

/// SHA-256 of `data` as lower-case hexadecimal.
#[must_use]
pub fn sha256_hex(data: &[u8]) -> String {
    hex(&Sha256::digest(data))
}

/// HMAC-SHA-256 of `message` under `key`.
#[must_use]
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    // HMAC accepts keys of any length, so `new_from_slice` cannot fail; the fallback keeps
    // this function total without a panic path.
    let Ok(mut mac) = HmacSha256::new_from_slice(key) else { return [0u8; 32] };
    mac.update(message);
    mac.finalize().into_bytes().into()
}

/// Constant-time equality of two byte strings (length is not secret).
#[must_use]
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && bool::from(a.ct_eq(b))
}

/// The stored form of a gift code: hex HMAC-SHA-256 of its compact form under the server
/// secret. The clear code exists only in the creation response and on the card.
#[must_use]
pub fn gift_code_hash(secret: &[u8], code: &GiftCode) -> String {
    hex(&hmac_sha256(secret, format!("ebc:gift-code:v1:{}", code.compact()).as_bytes()))
}

/// The stored form of a pair code.
#[must_use]
pub fn pair_code_hash(secret: &[u8], code: &PairCode) -> String {
    hex(&hmac_sha256(secret, format!("ebc:pair-code:v1:{}", code.compact()).as_bytes()))
}

/// The stored form of a bearer token (manage token).
#[must_use]
pub fn token_hash(token: &str) -> String {
    sha256_hex(format!("ebc:token:v1:{token}").as_bytes())
}

/// `n` random bytes as hexadecimal (`2n` characters).
///
/// # Errors
/// When the random source fails.
pub fn random_hex(entropy: &dyn Entropy, n: usize) -> Result<String, EntropyError> {
    let mut bytes = vec![0u8; n];
    entropy.fill(&mut bytes)?;
    Ok(hex(&bytes))
}

/// The code a partner gift's card carries, re-derivable from a stored random nonce.
///
/// Partner gifts must be renderable server-side after creation
/// (`GET /api/partner/v1/gifts/{id}/card.pdf`), yet the clear code is never stored. The
/// code is therefore `GiftCode::from_random_bytes(HMAC(secret, nonce ‖ counter))` for a
/// random 128-bit nonce kept with the gift: without the server secret the nonce reveals
/// nothing (HMAC is a pseudo-random function), exactly like the stored code hash.
/// `counter` skips the rare candidates rejected by
/// [`GiftCode::has_unfortunate_letters`]; it is returned so the caller can store it.
///
/// Returns `None` only if the model refuses 32 random bytes, which it never does.
#[must_use]
pub fn derive_gift_code(secret: &[u8], nonce_hex: &str, start_counter: u32) -> Option<(GiftCode, u32)> {
    (start_counter..start_counter.saturating_add(64)).find_map(|counter| {
        let material =
            hmac_sha256(secret, format!("ebc:derived-gift-code:v1:{nonce_hex}:{counter}").as_bytes());
        let code = GiftCode::from_random_bytes(&material).ok()?;
        (!code.has_unfortunate_letters()).then_some((code, counter))
    })
}

/// A fresh random gift code (never one flagged by `has_unfortunate_letters`).
///
/// # Errors
/// When the random source fails.
pub fn random_gift_code(entropy: &dyn Entropy) -> Result<GiftCode, EntropyError> {
    for _ in 0..64 {
        let mut bytes = [0u8; 32];
        entropy.fill(&mut bytes)?;
        if let Ok(code) = GiftCode::from_random_bytes(&bytes) {
            if !code.has_unfortunate_letters() {
                return Ok(code);
            }
        }
    }
    Err(EntropyError("could not draw an acceptable gift code".to_owned()))
}

/// A fresh random pair code.
///
/// # Errors
/// When the random source fails.
pub fn random_pair_code(entropy: &dyn Entropy) -> Result<PairCode, EntropyError> {
    let mut bytes = [0u8; 32];
    entropy.fill(&mut bytes)?;
    PairCode::from_random_bytes(&bytes).map_err(|e| EntropyError(e.to_string()))
}

/// The string a partner signs for an API request, and the server for a webhook (§4.1):
/// `"{timestamp}.{METHOD}.{path-with-query}.{hex sha256(body)}"`.
#[must_use]
pub fn request_signing_string(timestamp: i64, method: &str, path_and_query: &str, body: &[u8]) -> String {
    format!("{timestamp}.{}.{path_and_query}.{}", method.to_ascii_uppercase(), sha256_hex(body))
}

/// The `X-EBC-Signature` header value for a request: `v1=<hex>`.
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

/// Verifies an `X-EBC-Signature` header value in constant time.
#[must_use]
pub fn verify_request_signature(
    secret: &str,
    timestamp: i64,
    method: &str,
    path_and_query: &str,
    body: &[u8],
    header_value: &str,
) -> bool {
    let Some(provided) = header_value.trim().strip_prefix("v1=").and_then(unhex) else {
        return false;
    };
    let message = request_signing_string(timestamp, method, path_and_query, body);
    ct_eq(&hmac_sha256(secret.as_bytes(), message.as_bytes()), &provided)
}

/// The `ebc_sig` parameter of a redemption redirect (§4.3): hex HMAC-SHA-256 of
/// `"{gift_id}.{order_ref}.{ts}"` under the partner secret.
#[must_use]
pub fn redirect_signature(secret: &str, gift_id: &str, order_ref: &str, ts: i64) -> String {
    hex(&hmac_sha256(secret.as_bytes(), format!("{gift_id}.{order_ref}.{ts}").as_bytes()))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::clock::OsEntropy;

    #[test]
    fn hex_round_trips_and_rejects_garbage() {
        assert_eq!(hex(&[0x00, 0x0f, 0xa5, 0xff]), "000fa5ff");
        assert_eq!(unhex("000FA5ff"), Some(vec![0x00, 0x0f, 0xa5, 0xff]));
        assert_eq!(unhex(""), Some(Vec::new()));
        for bad in ["0", "0g", "é0", " 00", "0x00"] {
            assert_eq!(unhex(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn hmac_matches_rfc_4231_test_case_2() {
        let mac = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(hex(&mac), "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn code_hashes_depend_on_secret_code_and_kind() {
        let code = GiftCode::from_random_bytes(&[7u8; 16]).unwrap();
        let other = GiftCode::from_random_bytes(&[8u8; 16]).unwrap();
        let hash = gift_code_hash(b"secret-a", &code);
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, gift_code_hash(b"secret-a", &code));
        assert_ne!(hash, gift_code_hash(b"secret-b", &code));
        assert_ne!(hash, gift_code_hash(b"secret-a", &other));
        assert!(!hash.contains(&code.compact().to_lowercase()));
        let pair = PairCode::from_random_bytes(&[9u8; 16]).unwrap();
        assert_ne!(pair_code_hash(b"secret-a", &pair), pair_code_hash(b"secret-b", &pair));
        assert_ne!(token_hash("a"), token_hash("b"));
        assert!(!token_hash("my-token").contains("my-token"));
    }

    #[test]
    fn derived_codes_are_stable_and_secret_dependent() {
        let (code, counter) = derive_gift_code(b"secret", "00112233445566778899aabbccddeeff", 0).unwrap();
        let (again, same) = derive_gift_code(b"secret", "00112233445566778899aabbccddeeff", counter).unwrap();
        assert_eq!((code, counter), (again, same));
        assert!(!code.has_unfortunate_letters());
        let (other, _) = derive_gift_code(b"other-secret", "00112233445566778899aabbccddeeff", 0).unwrap();
        assert_ne!(code, other);
        let (third, _) = derive_gift_code(b"secret", "ffffffffffffffffffffffffffffffff", 0).unwrap();
        assert_ne!(code, third);
        // Derived codes are well-formed codes.
        assert_eq!(GiftCode::parse(&code.grouped()), Ok(code));
    }

    #[test]
    fn random_values_are_distinct_and_well_formed() {
        let a = random_gift_code(&OsEntropy).unwrap();
        let b = random_gift_code(&OsEntropy).unwrap();
        assert_ne!(a, b);
        assert_eq!(a.compact().len(), GiftCode::LEN);
        let p = random_pair_code(&OsEntropy).unwrap();
        assert_eq!(p.compact().len(), PairCode::LEN);
        let token = random_hex(&OsEntropy, 16).unwrap();
        assert_eq!(token.len(), 32);
        assert!(token.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_ne!(token, random_hex(&OsEntropy, 16).unwrap());
    }

    #[test]
    fn a_failing_random_source_is_an_error_not_a_weak_code() {
        struct Broken;
        impl Entropy for Broken {
            fn fill(&self, _: &mut [u8]) -> Result<(), EntropyError> {
                Err(EntropyError("no entropy".into()))
            }
        }
        assert!(random_gift_code(&Broken).is_err());
        assert!(random_pair_code(&Broken).is_err());
        assert!(random_hex(&Broken, 16).is_err());
    }

    #[test]
    fn request_signatures_follow_the_contract() {
        let body = br#"{"order_ref":"ORD-1"}"#;
        let signing = request_signing_string(1_800_000_000, "post", "/api/partner/v1/gifts?x=1", body);
        assert_eq!(
            signing,
            format!("1800000000.POST./api/partner/v1/gifts?x=1.{}", sha256_hex(body))
        );
        let header = request_signature("s3cret", 1_800_000_000, "POST", "/api/partner/v1/gifts?x=1", body);
        assert!(header.starts_with("v1=") && header.len() == 3 + 64);
        let verify = |secret: &str, ts: i64, method: &str, path: &str, body: &[u8], value: &str| {
            verify_request_signature(secret, ts, method, path, body, value)
        };
        assert!(verify("s3cret", 1_800_000_000, "POST", "/api/partner/v1/gifts?x=1", body, &header));
        assert!(verify("s3cret", 1_800_000_000, "post", "/api/partner/v1/gifts?x=1", body, &header.to_uppercase().replacen("V1=", "v1=", 1)));
        assert!(!verify("other", 1_800_000_000, "POST", "/api/partner/v1/gifts?x=1", body, &header));
        assert!(!verify("s3cret", 1_800_000_001, "POST", "/api/partner/v1/gifts?x=1", body, &header));
        assert!(!verify("s3cret", 1_800_000_000, "PUT", "/api/partner/v1/gifts?x=1", body, &header));
        assert!(!verify("s3cret", 1_800_000_000, "POST", "/api/partner/v1/gifts", body, &header));
        assert!(!verify("s3cret", 1_800_000_000, "POST", "/api/partner/v1/gifts?x=1", b"{}", &header));
        for malformed in ["", "v1=", "v2=00", &header[3..], "v1=zz", "v1=00"] {
            assert!(
                !verify("s3cret", 1_800_000_000, "POST", "/api/partner/v1/gifts?x=1", body, malformed),
                "{malformed:?}"
            );
        }
    }

    #[test]
    fn redirect_signature_signs_id_order_and_time() {
        let sig = redirect_signature("s3cret", "g_1", "ORD-1", 1_800_000_000);
        assert_eq!(sig, hex(&hmac_sha256(b"s3cret", b"g_1.ORD-1.1800000000")));
        assert_ne!(sig, redirect_signature("s3cret", "g_1", "ORD-1", 1_800_000_001));
        assert_ne!(sig, redirect_signature("s3cret", "g_2", "ORD-1", 1_800_000_000));
        assert_ne!(sig, redirect_signature("other", "g_1", "ORD-1", 1_800_000_000));
    }
}
