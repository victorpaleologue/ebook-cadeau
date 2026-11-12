//! Minimal ICC profile header inspection.
//!
//! The writer embeds ICC profiles verbatim; it only reads the 128-byte header to make
//! sure a profile is what the document says it is (component count, version supported by
//! the target PDF version). Callers can use [`inspect`] for their own checks — for
//! example to warn when an output-intent profile is not a printer (`prtr`) profile.

/// What the header of an ICC profile says about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IccInfo {
    /// Number of colour components of the profile's data colour space
    /// (1 = grey, 3 = RGB, 4 = CMYK).
    pub components: u8,
    /// Major version of the ICC specification the profile follows (2 or 4).
    pub version_major: u8,
    /// Minor version (the high nibble of the header's minor/bug-fix byte).
    pub version_minor: u8,
    /// Profile/device class signature, e.g. `*b"prtr"` (output device), `*b"mntr"`
    /// (display), `*b"scnr"` (input device).
    pub device_class: [u8; 4],
    /// Data colour space signature, e.g. `*b"CMYK"`, `*b"RGB "`, `*b"GRAY"`.
    pub color_space: [u8; 4],
}

impl IccInfo {
    /// `true` for an output-device (printer) profile, the class PDF/X expects for the
    /// output intent's `DestOutputProfile`.
    #[must_use]
    pub fn is_output_class(&self) -> bool {
        &self.device_class == b"prtr"
    }
}

/// Reads the header of an ICC profile.
///
/// # Errors
/// Returns a short description when the data is not an ICC profile or uses a data colour
/// space other than grey, RGB or CMYK.
pub fn inspect(profile: &[u8]) -> Result<IccInfo, &'static str> {
    if profile.len() < 128 {
        return Err("shorter than the 128-byte ICC header");
    }
    if &profile[36..40] != b"acsp" {
        return Err("missing the 'acsp' signature");
    }
    let declared = u32::from_be_bytes([profile[0], profile[1], profile[2], profile[3]]);
    let declared = usize::try_from(declared).unwrap_or(usize::MAX);
    if declared < 128 || declared > profile.len() {
        return Err("the size field does not match the data length");
    }
    let mut device_class = [0u8; 4];
    device_class.copy_from_slice(&profile[12..16]);
    let mut color_space = [0u8; 4];
    color_space.copy_from_slice(&profile[16..20]);
    let components = match &color_space {
        b"GRAY" => 1,
        b"RGB " => 3,
        b"CMYK" => 4,
        _ => return Err("data colour space is not grey, RGB or CMYK"),
    };
    Ok(IccInfo {
        components,
        version_major: profile[8],
        version_minor: profile[9] >> 4,
        device_class,
        color_space,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A syntactically valid header with nothing behind it.
    fn header(space: &[u8; 4], class: &[u8; 4], major: u8) -> Vec<u8> {
        let mut p = vec![0u8; 128];
        p[0..4].copy_from_slice(&128u32.to_be_bytes());
        p[8] = major;
        p[9] = 0x20;
        p[12..16].copy_from_slice(class);
        p[16..20].copy_from_slice(space);
        p[36..40].copy_from_slice(b"acsp");
        p
    }

    #[test]
    fn reads_the_header_fields() {
        let info = inspect(&header(b"CMYK", b"prtr", 2)).expect("valid");
        assert_eq!(info.components, 4);
        assert_eq!((info.version_major, info.version_minor), (2, 2));
        assert!(info.is_output_class());
        let info = inspect(&header(b"RGB ", b"mntr", 4)).expect("valid");
        assert_eq!(info.components, 3);
        assert!(!info.is_output_class());
        assert_eq!(inspect(&header(b"GRAY", b"prtr", 2)).expect("valid").components, 1);
    }

    #[test]
    fn rejects_garbage() {
        assert!(inspect(&[]).is_err());
        assert!(inspect(&[0u8; 127]).is_err());
        assert!(inspect(&[0u8; 500]).is_err());
        assert!(inspect(&header(b"Lab ", b"abst", 4)).is_err());
        let mut truncated = header(b"CMYK", b"prtr", 2);
        truncated[0..4].copy_from_slice(&4096u32.to_be_bytes());
        assert!(inspect(&truncated).is_err());
        let mut tiny = header(b"CMYK", b"prtr", 2);
        tiny[0..4].copy_from_slice(&5u32.to_be_bytes());
        assert!(inspect(&tiny).is_err());
    }

    #[test]
    fn bundled_profiles_are_what_they_claim() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/icc/");
        let srgb = std::fs::read(format!("{dir}sRGB-v2-micro.icc")).expect("asset");
        let info = inspect(&srgb).expect("valid");
        assert_eq!((info.components, info.version_major), (3, 2));
        let cmyk = std::fs::read(format!("{dir}CGATS001Compat-v2-micro.icc")).expect("asset");
        let info = inspect(&cmyk).expect("valid");
        assert_eq!((info.components, info.version_major), (4, 2));
        // Documented caveat: the compact CMYK profile is an *input* class profile.
        assert!(!info.is_output_class());
    }
}
