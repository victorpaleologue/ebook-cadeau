//! A minimal owned RGBA raster, used to pass decoded images between crates
//! without forcing an image-decoding dependency on any of them.

/// An 8-bit-per-channel, non-premultiplied RGBA image in row-major order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pixmap {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl Pixmap {
    /// Wraps an RGBA8 buffer. Returns `None` if `data.len() != width * height * 4`
    /// or if either dimension is zero.
    #[must_use]
    pub fn from_rgba8(width: u32, height: u32, data: Vec<u8>) -> Option<Self> {
        let expected = (width as usize).checked_mul(height as usize)?.checked_mul(4)?;
        (width > 0 && height > 0 && data.len() == expected).then_some(Self { width, height, data })
    }

    /// Creates an image filled with one colour.
    ///
    /// # Panics
    /// Panics if either dimension is zero or the byte size overflows `usize`.
    #[must_use]
    pub fn filled(width: u32, height: u32, rgba: [u8; 4]) -> Self {
        assert!(width > 0 && height > 0, "Pixmap dimensions must be non-zero");
        let n = (width as usize).checked_mul(height as usize).expect("pixmap size overflow");
        let mut data = Vec::with_capacity(n.checked_mul(4).expect("pixmap size overflow"));
        for _ in 0..n {
            data.extend_from_slice(&rgba);
        }
        Self { width, height, data }
    }

    /// Width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The raw RGBA8 bytes, row-major, 4 bytes per pixel.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Mutable access to the raw RGBA8 bytes.
    #[must_use]
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Consumes the image and returns its RGBA8 bytes.
    #[must_use]
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    /// The pixel at `(x, y)`, or `None` if out of bounds.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let i = (y as usize * self.width as usize + x as usize) * 4;
        Some([self.data[i], self.data[i + 1], self.data[i + 2], self.data[i + 3]])
    }

    /// Sets the pixel at `(x, y)`. Out-of-bounds coordinates are ignored.
    pub fn set(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x < self.width && y < self.height {
            let i = (y as usize * self.width as usize + x as usize) * 4;
            self.data[i..i + 4].copy_from_slice(&rgba);
        }
    }

    /// Returns `true` if every pixel is fully opaque.
    #[must_use]
    pub fn is_opaque(&self) -> bool {
        self.data.chunks_exact(4).all(|p| p[3] == 255)
    }

    /// Copies the rectangle starting at `(x, y)` with size `(w, h)`, clamped to the image.
    /// Returns `None` if the clamped rectangle is empty.
    #[must_use]
    pub fn crop(&self, x: u32, y: u32, w: u32, h: u32) -> Option<Self> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let w = w.min(self.width - x);
        let h = h.min(self.height - y);
        if w == 0 || h == 0 {
            return None;
        }
        let mut data = Vec::with_capacity(w as usize * h as usize * 4);
        for row in y..y + h {
            let start = (row as usize * self.width as usize + x as usize) * 4;
            data.extend_from_slice(&self.data[start..start + w as usize * 4]);
        }
        Some(Self { width: w, height: h, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construction_validates_length() {
        assert!(Pixmap::from_rgba8(2, 2, vec![0; 16]).is_some());
        assert!(Pixmap::from_rgba8(2, 2, vec![0; 15]).is_none());
        assert!(Pixmap::from_rgba8(0, 2, vec![]).is_none());
    }

    #[test]
    fn get_set_crop() {
        let mut p = Pixmap::filled(4, 3, [1, 2, 3, 255]);
        assert!(p.is_opaque());
        p.set(3, 2, [9, 9, 9, 0]);
        assert_eq!(p.get(3, 2), Some([9, 9, 9, 0]));
        assert_eq!(p.get(4, 0), None);
        assert!(!p.is_opaque());
        let c = p.crop(2, 1, 10, 10).expect("non-empty");
        assert_eq!((c.width(), c.height()), (2, 2));
        assert_eq!(c.get(1, 1), Some([9, 9, 9, 0]));
        assert!(p.crop(4, 0, 1, 1).is_none());
    }
}
