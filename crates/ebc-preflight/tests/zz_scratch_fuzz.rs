#![allow(clippy::unwrap_used)]
mod common;
use common::{Fixture, Rng, card_expectations};
use ebc_preflight::{PreflightError, check};
use std::time::Instant;

#[test]
fn campaign() {
    let mut seeds: Vec<Vec<u8>> = Vec::new();
    let mut f = Fixture::reference();
    seeds.push(f.bytes());
    seeds.push(f.bytes_modern());
    for name in ["card-x4.pdf", "card-x1a.pdf", "card-plain.pdf", "card-x4-uncompressed.pdf"] {
        let p = format!("{}/../../target/tmp/ebc-pdf/sample-card/{name}", env!("CARGO_MANIFEST_DIR"));
        if let Ok(b) = std::fs::read(&p) { seeds.push(b); }
    }
    eprintln!("{} seeds", seeds.len());
    let expect = card_expectations();
    let mut rng = Rng::new(0xF00D_0001);
    let (mut internal, mut ok, mut err) = (0, 0, 0);
    let mut slowest = (0.0_f64, 0usize, 0usize);
    let rounds: usize = std::env::var("ROUNDS").ok().and_then(|v| v.parse().ok()).unwrap_or(2000);
    for round in 0..rounds {
        let si = round % seeds.len();
        let mut data = seeds[si].clone();
        match rng.below(6) {
            0 => { let n = 1 + rng.below(6); for _ in 0..n { let at = rng.below(data.len()); data[at] = rng.bytes(1)[0]; } }
            1 => { let cut = rng.below(data.len()); data.truncate(cut); }
            2 => { let at = rng.below(data.len()); let len = (1 + rng.below(2000)).min(data.len() - at); data.drain(at..at + len); }
            3 => { let at = rng.below(data.len()); let len = (1 + rng.below(300)).min(data.len() - at); let chunk = data[at..at+len].to_vec(); let to = rng.below(data.len()); for (i, b) in chunk.into_iter().enumerate() { data.insert(to + i, b); } }
            4 => { // digit tweak: change a random ASCII digit
                for _ in 0..50 { let at = rng.below(data.len()); if data[at].is_ascii_digit() { data[at] = b'0' + (rng.below(10) as u8); break; } } }
            _ => { let at = rng.below(data.len()); let len = (1 + rng.below(64)).min(data.len() - at); for b in &mut data[at..at+len] { *b ^= 0xFF; } }
        }
        let t = Instant::now();
        match check(&data, &expect) {
            Ok(r) => { ok += 1; let _ = r.to_json(); }
            Err(PreflightError::Internal(m)) => { internal += 1; if internal < 10 { eprintln!("INTERNAL round {round} seed {si}: {m}"); std::fs::write(format!("/tmp/claude-0/-home-user-ebook-cadeau/cb53cea9-c2e9-5296-89ff-c6ff897e1e73/scratchpad/internal-{round}.pdf"), &data).ok(); } }
            Err(_) => err += 1,
        }
        let dt = t.elapsed().as_secs_f64();
        if dt > slowest.0 { slowest = (dt, round, si); }
    }
    eprintln!("ok {ok} err {err} internal {internal} slowest {slowest:?}");
    assert_eq!(internal, 0);
}
