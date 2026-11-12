//! Optimal splitting of a text into numeric, alphanumeric and byte segments.
//!
//! A QR code stores its payload as a sequence of segments, each in one encoding mode:
//!
//! | mode         | characters                                  | cost per character |
//! |--------------|---------------------------------------------|--------------------|
//! | numeric      | `0`–`9`                                     | 3⅓ bits            |
//! | alphanumeric | `0`–`9`, `A`–`Z`, space, `$ % * + - . / :`  | 5½ bits            |
//! | byte         | anything (as UTF-8)                         | 8 bits per byte    |
//!
//! plus a header of 4 bits and a length field per segment. The encoder this crate wraps
//! picks one mode for the whole text; this module finds the cheapest *mix* by dynamic
//! programming (the method of Project Nayuki's "optimal text segmentation"), which is what
//! lets `HTTPS://EBK.EXAMPLE/k/0123456789` use the compact modes around its lower-case
//! part. Kanji mode is not used.

/// The character set of alphanumeric mode, in code order.
pub(crate) const ALPHANUMERIC_CHARSET: &str = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";

/// A segment encoding mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Digits only.
    Numeric,
    /// The 45-character alphanumeric set.
    Alphanumeric,
    /// Raw bytes (UTF-8).
    Byte,
}

/// The modes in the order used for tie-breaking (cheapest per character first).
const MODES: [Mode; 3] = [Mode::Numeric, Mode::Alphanumeric, Mode::Byte];

/// The three ranges of versions that share the same length-field widths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VersionGroup {
    /// Versions 1–9.
    Small,
    /// Versions 10–26.
    Medium,
    /// Versions 27–40.
    Large,
}

impl VersionGroup {
    /// All groups, smallest versions first.
    pub(crate) const ALL: [Self; 3] = [Self::Small, Self::Medium, Self::Large];

    /// The inclusive range of versions in the group.
    pub(crate) fn versions(self) -> (u8, u8) {
        match self {
            Self::Small => (1, 9),
            Self::Medium => (10, 26),
            Self::Large => (27, 40),
        }
    }
}

impl Mode {
    /// Whether `c` can be encoded in this mode.
    pub(crate) fn accepts(self, c: char) -> bool {
        match self {
            Self::Numeric => c.is_ascii_digit(),
            Self::Alphanumeric => ALPHANUMERIC_CHARSET.contains(c),
            Self::Byte => true,
        }
    }

    /// Width in bits of the segment's character-count field (ISO/IEC 18004, table 3).
    pub(crate) fn char_count_bits(self, group: VersionGroup) -> u32 {
        let widths = match self {
            Self::Numeric => [10, 12, 14],
            Self::Alphanumeric => [9, 11, 13],
            Self::Byte => [8, 16, 16],
        };
        widths[group as usize]
    }

    /// Cost of one character in sixths of a bit (the common denominator of 3⅓, 5½ and 8).
    fn char_cost_sixths(self, c: char) -> u64 {
        match self {
            Self::Numeric => 20,
            Self::Alphanumeric => 33,
            Self::Byte => 48 * c.len_utf8() as u64,
        }
    }

    /// Cost of a segment header (mode indicator + length field) in sixths of a bit.
    fn header_cost_sixths(self, group: VersionGroup) -> u64 {
        6 * (4 + u64::from(self.char_count_bits(group)))
    }

    fn index(self) -> usize {
        match self {
            Self::Numeric => 0,
            Self::Alphanumeric => 1,
            Self::Byte => 2,
        }
    }
}

/// A run of consecutive characters encoded in one mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Run<'a> {
    pub(crate) mode: Mode,
    pub(crate) text: &'a str,
}

/// Rounds a cost in sixths of a bit up to a whole number of bits (still in sixths).
fn ceil_to_bit(sixths: u64) -> u64 {
    sixths.div_ceil(6) * 6
}

/// Splits `text` into the sequence of runs with the smallest total bit length for the
/// given version group. Returns no runs for an empty text.
///
/// Ties are broken deterministically: stay in the current mode rather than switch, and
/// otherwise prefer numeric over alphanumeric over byte.
pub(crate) fn optimal_runs(text: &str, group: VersionGroup) -> Vec<Run<'_>> {
    const UNREACHABLE: u64 = u64::MAX / 4;
    if text.is_empty() {
        return Vec::new();
    }
    let headers = MODES.map(|m| m.header_cost_sixths(group));

    // open[m]: cheapest cost of everything so far, given that a segment of mode `m` is
    // open (its header paid) and the next character will go into it.
    let mut open = headers;
    // used[i][m]: the mode character `i` was encoded in on the cheapest way to `open[m]`
    // after character `i`.
    let mut used: Vec<[Option<Mode>; 3]> = Vec::with_capacity(text.len());

    for c in text.chars() {
        // Append the character to the open segment of each mode that can hold it.
        let stay = MODES.map(|m| {
            if m.accepts(c) { open[m.index()] + m.char_cost_sixths(c) } else { UNREACHABLE }
        });
        let mut next = stay;
        let mut via: [Option<Mode>; 3] =
            MODES.map(|m| (stay[m.index()] < UNREACHABLE).then_some(m));
        // Or close the segment after this character and open one of another mode.
        for target in MODES {
            for source in MODES {
                if source == target || stay[source.index()] >= UNREACHABLE {
                    continue;
                }
                let cost = ceil_to_bit(stay[source.index()]) + headers[target.index()];
                if cost < next[target.index()] {
                    next[target.index()] = cost;
                    via[target.index()] = Some(source);
                }
            }
        }
        open = next;
        used.push(via);
    }

    // Walk back from the cheapest final state.
    let mut state = MODES[0];
    for m in MODES {
        if open[m.index()] < open[state.index()] {
            state = m;
        }
    }
    let mut modes: Vec<Mode> = vec![Mode::Byte; used.len()];
    for (slot, via) in modes.iter_mut().zip(&used).rev() {
        // Byte mode accepts every character, so every state is reachable; the fallback
        // is never taken.
        state = via[state.index()].unwrap_or(Mode::Byte);
        *slot = state;
    }

    // Merge consecutive characters of the same mode into runs. `char_indices` only yields
    // character boundaries, so the slices below cannot fail.
    let mut runs: Vec<Run<'_>> = Vec::new();
    let mut start = 0;
    let mut current: Option<Mode> = None;
    for ((offset, _), mode) in text.char_indices().zip(modes) {
        match current {
            Some(open_mode) if open_mode != mode => {
                runs.push(Run { mode: open_mode, text: &text[start..offset] });
                start = offset;
            }
            _ => {}
        }
        current = Some(mode);
    }
    if let Some(mode) = current {
        runs.push(Run { mode, text: &text[start..] });
    }
    runs
}

/// Exact number of data bits of one segment holding `chars` characters (`bytes` UTF-8
/// bytes) in `mode`, header included.
#[cfg(test)]
pub(crate) fn segment_bits(mode: Mode, chars: usize, bytes: usize, group: VersionGroup) -> u64 {
    let (chars, bytes) = (chars as u64, bytes as u64);
    let data = match mode {
        Mode::Numeric => 10 * (chars / 3) + [0, 4, 7][(chars % 3) as usize],
        Mode::Alphanumeric => 11 * (chars / 2) + 6 * (chars % 2),
        Mode::Byte => 8 * bytes,
    };
    4 + u64::from(mode.char_count_bits(group)) + data
}

/// Total number of data bits of a sequence of runs.
#[cfg(test)]
pub(crate) fn total_bits(runs: &[Run<'_>], group: VersionGroup) -> u64 {
    runs.iter()
        .map(|run| segment_bits(run.mode, run.text.chars().count(), run.text.len(), group))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modes_of(text: &str, group: VersionGroup) -> Vec<(Mode, &str)> {
        optimal_runs(text, group).into_iter().map(|r| (r.mode, r.text)).collect()
    }

    /// Cheapest total over every assignment of a mode to every character (3ⁿ of them),
    /// computed independently of the dynamic programme.
    fn brute_force_bits(text: &str, group: VersionGroup) -> u64 {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let mut best = u64::MAX;
        for code in 0..3usize.pow(n as u32) {
            let mut assignment = Vec::with_capacity(n);
            let mut rest = code;
            for _ in 0..n {
                assignment.push(MODES[rest % 3]);
                rest /= 3;
            }
            if chars.iter().zip(&assignment).any(|(&c, m)| !m.accepts(c)) {
                continue;
            }
            let mut total = 0;
            let mut start = 0;
            while start < n {
                let mode = assignment[start];
                let mut end = start;
                while end < n && assignment[end] == mode {
                    end += 1;
                }
                let bytes: usize = chars[start..end].iter().map(|c| c.len_utf8()).sum();
                total += segment_bits(mode, end - start, bytes, group);
                start = end;
            }
            best = best.min(total);
        }
        best
    }

    #[test]
    fn single_mode_texts_stay_in_one_segment() {
        for group in VersionGroup::ALL {
            assert_eq!(modes_of("0123456789", group), [(Mode::Numeric, "0123456789")]);
            assert_eq!(modes_of("HELLO WORLD", group), [(Mode::Alphanumeric, "HELLO WORLD")]);
            assert_eq!(modes_of("hello, world", group), [(Mode::Byte, "hello, world")]);
            assert_eq!(modes_of("ça €", group), [(Mode::Byte, "ça €")]);
            assert!(optimal_runs("", group).is_empty());
        }
    }

    #[test]
    fn mixed_texts_switch_only_when_it_pays() {
        let g = VersionGroup::Small;
        // 20 digits after an alphanumeric prefix: 14 + 67 bits numeric beats 110 bits.
        assert_eq!(
            modes_of("HTTPS://EBK.EXAMPLE/K/01234567890123456789", g),
            [
                (Mode::Alphanumeric, "HTTPS://EBK.EXAMPLE/K/"),
                (Mode::Numeric, "01234567890123456789")
            ]
        );
        // 4 digits are not worth a 14-bit header: they stay alphanumeric.
        assert_eq!(modes_of("ABC/1234", g), [(Mode::Alphanumeric, "ABC/1234")]);
        // The eBook Cadeau claim URL with its lower-case path marker.
        // (123 + 20 + 74 = 217 bits; keeping the second "/" in byte mode would cost 219,
        // a single byte segment to the end 231.)
        assert_eq!(
            modes_of("HTTPS://EBK.EXAMPLE/k/7K3M9QX2RD", g),
            [
                (Mode::Alphanumeric, "HTTPS://EBK.EXAMPLE/"),
                (Mode::Byte, "k"),
                (Mode::Alphanumeric, "/7K3M9QX2RD")
            ]
        );
        // A short upper-case island inside lower-case text is absorbed by byte mode.
        assert_eq!(modes_of("abcDEFghi", g), [(Mode::Byte, "abcDEFghi")]);
    }

    #[test]
    fn length_field_width_changes_the_decision() {
        // Eight digits after an alphanumeric prefix. Versions 1–9: a numeric segment costs
        // 14 + 27 = 41 bits against 44 bits for staying alphanumeric → switch (71 < 74 in
        // total). Versions 27–40: its header is 4 bits longer → stay (78 < 79).
        let text = "AB:12345678";
        assert_eq!(
            modes_of(text, VersionGroup::Small),
            [(Mode::Alphanumeric, "AB:"), (Mode::Numeric, "12345678")]
        );
        assert_eq!(modes_of(text, VersionGroup::Large), [(Mode::Alphanumeric, text)]);
    }

    #[test]
    fn runs_tile_the_text_on_character_boundaries() {
        let text = "Zoë 2024: 0123456789012 — ÉTÉ/HIVER 🎁 abc";
        for group in VersionGroup::ALL {
            let runs = optimal_runs(text, group);
            let rebuilt: String = runs.iter().map(|run| run.text).collect();
            assert_eq!(rebuilt, text, "runs must tile the text in order");
            for run in &runs {
                assert!(!run.text.is_empty());
                assert!(run.text.chars().all(|c| run.mode.accepts(c)), "{run:?}");
            }
            for pair in runs.windows(2) {
                assert_ne!(pair[0].mode, pair[1].mode, "adjacent runs must differ");
            }
        }
    }

    #[test]
    fn dynamic_programme_is_optimal_against_brute_force() {
        // Characters chosen to exercise every mode, including multi-byte UTF-8.
        let alphabet: Vec<char> = "0123456789AZ $:/azé€".chars().collect();
        let mut state = 0x2545_F491_4F6C_DD1D_u64;
        let mut next = |bound: usize| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state % bound as u64) as usize
        };
        for _ in 0..250 {
            let len = 1 + next(8);
            let text: String = (0..len).map(|_| alphabet[next(alphabet.len())]).collect();
            for group in VersionGroup::ALL {
                let runs = optimal_runs(&text, group);
                assert_eq!(
                    total_bits(&runs, group),
                    brute_force_bits(&text, group),
                    "{text:?} in {group:?}: {runs:?}"
                );
            }
        }
        // Digit-heavy texts, where the numeric/alphanumeric trade-off is tight.
        for _ in 0..150 {
            let len = 5 + next(4);
            let text: String =
                (0..len).map(|_| if next(4) == 0 { 'A' } else { alphabet[next(10)] }).collect();
            for group in VersionGroup::ALL {
                let runs = optimal_runs(&text, group);
                assert_eq!(
                    total_bits(&runs, group),
                    brute_force_bits(&text, group),
                    "{text:?} in {group:?}: {runs:?}"
                );
            }
        }
    }

    #[test]
    fn segment_bit_counts_match_the_standard() {
        let g = VersionGroup::Small;
        // ISO/IEC 18004 examples: "01234567" numeric = 4 + 10 + 27 bits;
        // "AC-42" alphanumeric = 4 + 9 + 28 bits.
        assert_eq!(segment_bits(Mode::Numeric, 8, 8, g), 41);
        assert_eq!(segment_bits(Mode::Alphanumeric, 5, 5, g), 41);
        assert_eq!(segment_bits(Mode::Byte, 3, 5, g), 4 + 8 + 40);
        assert_eq!(segment_bits(Mode::Numeric, 1, 1, VersionGroup::Large), 4 + 14 + 4);
        assert_eq!(segment_bits(Mode::Numeric, 2, 2, VersionGroup::Medium), 4 + 12 + 7);
        assert_eq!(segment_bits(Mode::Byte, 1, 1, VersionGroup::Medium), 4 + 16 + 8);
    }

    #[test]
    fn alphanumeric_set_is_the_standard_one() {
        assert_eq!(ALPHANUMERIC_CHARSET.chars().count(), 45);
        for c in "abc#&@?=_~é\n\0".chars() {
            assert!(!Mode::Alphanumeric.accepts(c), "{c:?}");
            assert!(!Mode::Numeric.accepts(c));
            assert!(Mode::Byte.accepts(c));
        }
        assert!(Mode::Alphanumeric.accepts(' ') && Mode::Alphanumeric.accepts(':'));
        assert!(!Mode::Numeric.accepts('٣'), "non-ASCII digits are not numeric mode");
    }
}
