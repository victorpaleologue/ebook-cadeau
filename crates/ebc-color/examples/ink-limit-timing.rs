//! Times the ink-limited transform on a full poche cover (1240 × 2008 px, the sample
//! cover of `ebc-card`'s tests, whose navy frame and sky are above 240 %).
//!
//! ```sh
//! cargo run --release -p ebc-color --example ink-limit-timing
//! ```

use std::io::Write as _;
use std::time::Instant;

use ebc_color::{CmykProfile, InkLimit, SrgbToCmyk};

#[allow(dead_code, unreachable_pub)]
#[path = "../../ebc-card/tests/common/art.rs"]
mod art;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../ebc-card/assets/icc/ISOcoated_v2_300_bas.icc");
    let bytes = std::fs::read(path)?;
    let mut out = std::io::stdout().lock();
    let cover = art::rgb_bytes(&art::cover(1240, 2008, false, true));

    let t = Instant::now();
    let profile = CmykProfile::parse(&bytes)?;
    let plain = SrgbToCmyk::relative_bpc(&profile);
    let link_ms = t.elapsed().as_secs_f64() * 1e3;
    let t = Instant::now();
    let _ = plain.convert_pixels(&cover);
    let plain_px_ms = t.elapsed().as_secs_f64() * 1e3;

    for limit in [240.0_f32, 300.0] {
        let t = Instant::now();
        let limit_value = InkLimit::new(limit).ok_or("invalid limit")?;
        let limited = plain.clone().with_ink_limit(&profile, limit_value);
        let limit_ms = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let px = limited.convert_pixels(&cover);
        let px_ms = t.elapsed().as_secs_f64() * 1e3;
        let max = px.chunks_exact(4).map(|p| p.iter().map(|&v| u32::from(v)).sum::<u32>()).max();
        writeln!(
            out,
            "{limit} %: transform {link_ms:.0} ms + ink limit {limit_ms:.0} ms ({} nodes rebuilt); \
             cover 1240x2008: {px_ms:.0} ms (without limit {plain_px_ms:.0} ms); max ink {:.1} %",
            limited.rebuilt_nodes(),
            f64::from(max.unwrap_or(0)) * 100.0 / 255.0
        )?;
    }
    Ok(())
}
