//! An own-made cover for samples and tests: an abstract evening seascape drawn by a few
//! formulas. It reproduces nothing — no real book, publisher or artwork.

use ebc_scene::Pixmap;

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// The cover at `width` × `height` pixels. `light` gives a pale variant (pale sky, pale
/// frame), `frame` adds a plain border of 4 % of the width.
#[must_use]
pub fn cover(width: u32, height: u32, light: bool, frame: bool) -> Pixmap {
    let (w, h) = (f64::from(width.max(1)), f64::from(height.max(1)));
    let (top, bottom) = if light {
        ([236.0, 240.0, 246.0], [252.0, 226.0, 200.0])
    } else {
        ([18.0, 32.0, 74.0], [238.0, 120.0, 60.0])
    };
    let sun = if light { [250.0, 190.0, 90.0] } else { [255.0, 222.0, 128.0] };
    let border = if light { [248.0, 246.0, 240.0] } else { [14.0, 22.0, 48.0] };
    let mut data = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height.max(1) {
        for x in 0..width.max(1) {
            let (u, v) = (f64::from(x) / w, f64::from(y) / h);
            let mut c = mix(top, bottom, (v / 0.62).powf(1.4));
            // The sun, with a soft edge.
            let d = ((u - 0.5).powi(2) + ((v - 0.40) * h / w).powi(2)).sqrt();
            c = mix(sun, c, (d - 0.17) / 0.012);
            // Four bands of waves, darker towards the foot.
            for band in 0..4 {
                let b = f64::from(band);
                let crest = 0.60 + 0.085 * b
                    + 0.012 * ((u * (5.0 + 2.0 * b) + b * 1.7) * std::f64::consts::TAU).sin();
                if v > crest {
                    let sea = if light {
                        [150.0 - 22.0 * b, 180.0 - 20.0 * b, 205.0 - 14.0 * b]
                    } else {
                        [30.0 - 5.0 * b, 70.0 - 13.0 * b, 120.0 - 20.0 * b]
                    };
                    c = sea;
                }
            }
            if frame {
                let m = 0.04;
                if u < m || u > 1.0 - m || v < m * w / h || v > 1.0 - m * w / h {
                    c = border;
                }
            }
            data.extend_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
        }
    }
    Pixmap::from_rgba8(width.max(1), height.max(1), data)
        .unwrap_or_else(|| Pixmap::filled(1, 1, [0, 0, 0, 255]))
}

/// The pixels of a pixmap without their alpha channel.
#[must_use]
pub fn rgb_bytes(pixmap: &Pixmap) -> Vec<u8> {
    pixmap.data().chunks_exact(4).flat_map(|px| [px[0], px[1], px[2]]).collect()
}
