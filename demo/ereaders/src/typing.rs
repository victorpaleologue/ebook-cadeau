//! Keystroke-cost model for e-ink on-screen keyboards.
//!
//! Typing on an e-reader is slow (a tap, then a ~200–500 ms screen refresh), so the number of
//! taps is the UX currency of the "no phone" redemption path. This module turns a string into
//! a tap count so tests can hold the flow to a budget.
//!
//! # The model
//!
//! An on-screen keyboard has three **layers**: letters, digits + common punctuation, and
//! symbols. A text entry starts on the lower-case letters layer.
//!
//! | Action | Taps |
//! |---|---|
//! | a character on the current layer | 1 |
//! | letters ↔ digits, digits ↔ symbols | 1 (the layer stays until switched back) |
//! | letters ↔ symbols | 2 (through the digits layer) |
//! | one upper-case letter | +1 (one-shot shift) |
//! | a run of upper-case letters | caps lock when cheaper: +2 to lock (double tap), +1 to unlock if lower-case follows |
//! | space | 1 on every layer |
//! | a character on no layer (accents, `€`…) | 2 (long-press + pick), counted in `long_presses` |
//!
//! Which punctuation sits on the letters layer is a property of the [`Keyboard`]; a URL field
//! may expose extra keys (typically `/`). Keyboards with a number row have the digits on the
//! letters layer.
//!
//! **Everything about concrete devices' keyboards is an assumption** (no vendor documents its
//! key layout and the research could not verify any): see [`Keyboard`]'s constructors. The
//! model's value is relative — comparing two flows on the same keyboard — not absolute.

use std::ops::{Add, AddAssign};

/// Where the text is being typed; URL fields get a slightly different letters layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// A form field.
    Text,
    /// The browser's address bar.
    Url,
}

/// An on-screen keyboard: which characters are one tap away on which layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyboard {
    /// Display name.
    pub name: &'static str,
    /// Punctuation on the letters layer (space and a–z are always there).
    pub letters_extra: &'static str,
    /// Additional letters-layer keys when typing a URL.
    pub url_extra: &'static str,
    /// Digits on the letters layer (a dedicated number row).
    pub number_row: bool,
    /// Characters on the digits/punctuation layer.
    pub digits_layer: &'static str,
    /// Characters on the symbols layer.
    pub symbols_layer: &'static str,
    /// Whether shift can be locked for a run of capitals.
    pub caps_lock: bool,
}

impl Keyboard {
    /// A three-layer QWERTY/AZERTY-like keyboard without number row.
    ///
    /// *Assumption*: modelled on the common layout of Kobo and Kindle on-screen keyboards
    /// (letters; `.?123`-style digits layer; symbols layer), recalled from memory — not
    /// verified by the research (`ereader-ux.devices.md`, "Could not verify").
    pub fn three_layer() -> Self {
        Self {
            name: "three-layer",
            letters_extra: ".,",
            url_extra: "/",
            number_row: false,
            digits_layer: "0123456789-/:;()&@\".,?!'",
            symbols_layer: "[]{}#%^*+=_\\|~<>$",
            caps_lock: true,
        }
    }

    /// The same keyboard with a number row above the letters.
    ///
    /// *Assumption*: PocketBook-style keyboard with a digit row, recalled from memory.
    pub fn with_number_row() -> Self {
        Self { name: "number-row", number_row: true, ..Self::three_layer() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layer {
    Letters,
    Digits,
    Symbols,
}

/// The cost of typing something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TypingCost {
    /// Total taps, everything included.
    pub taps: u32,
    /// Characters typed.
    pub characters: u32,
    /// Taps spent switching layers.
    pub layer_switches: u32,
    /// Taps spent on shift / caps lock.
    pub shift_taps: u32,
    /// Characters that needed a long-press (not on any layer).
    pub long_presses: u32,
}

impl TypingCost {
    /// Cost of typing `text` exactly as given.
    pub fn of(text: &str, keyboard: &Keyboard, mode: InputMode) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let mut cost = Self::default();
        let mut layer = Layer::Letters;
        let mut caps_locked = false;

        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            cost.characters += 1;
            if c == ' ' {
                cost.taps += 1;
                i += 1;
                continue;
            }
            let target = layer_of(c, keyboard, mode, layer);
            match target {
                Some(target) => {
                    let switches = distance(layer, target);
                    cost.layer_switches += switches;
                    cost.taps += switches;
                    layer = target;
                }
                None => {
                    // Long-press on the letters layer (accented letters, currency signs…).
                    let switches = distance(layer, Layer::Letters);
                    cost.layer_switches += switches;
                    cost.taps += switches;
                    layer = Layer::Letters;
                    cost.long_presses += 1;
                    cost.taps += 1; // the pick after the long-press; the press itself is below
                }
            }
            if c.is_uppercase() {
                if !caps_locked {
                    let upper_ahead = chars[i..]
                        .iter()
                        .take_while(|c| !c.is_lowercase())
                        .filter(|c| c.is_uppercase())
                        .count();
                    let lower_follows = chars[i..].iter().any(|c| c.is_lowercase());
                    let lock_cost = 2 + usize::from(lower_follows);
                    if keyboard.caps_lock && upper_ahead > lock_cost {
                        caps_locked = true;
                        cost.shift_taps += 2;
                        cost.taps += 2;
                    } else {
                        cost.shift_taps += 1;
                        cost.taps += 1;
                    }
                }
            } else if c.is_lowercase() && caps_locked {
                caps_locked = false;
                cost.shift_taps += 1;
                cost.taps += 1;
            }
            cost.taps += 1;
            i += 1;
        }
        cost
    }

    /// Cost of typing a URL into the address bar.
    ///
    /// The scheme is not typed (*assumption*: e-reader address bars add `http://` themselves,
    /// as every mainstream browser does) and a bare trailing `/` is dropped. Host names are
    /// case-insensitive, so the host is typed in lower case; the path is typed as given.
    pub fn of_url(url: &str, keyboard: &Keyboard) -> Self {
        Self::of(&typed_url(url), keyboard, InputMode::Url)
    }
}

/// What a person actually types for `url`: no scheme, lower-case host, no bare trailing slash.
pub fn typed_url(url: &str) -> String {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .or_else(|| url.strip_prefix("HTTPS://"))
        .or_else(|| url.strip_prefix("HTTP://"))
        .unwrap_or(url);
    let (host, path) = match rest.find(['/', '?', '#']) {
        Some(at) => rest.split_at(at),
        None => (rest, ""),
    };
    let path = if path == "/" { "" } else { path };
    format!("{}{}", host.to_lowercase(), path)
}

fn layer_of(c: char, keyboard: &Keyboard, mode: InputMode, current: Layer) -> Option<Layer> {
    let on_letters = c.is_ascii_alphabetic()
        || keyboard.letters_extra.contains(c)
        || (mode == InputMode::Url && keyboard.url_extra.contains(c))
        || (keyboard.number_row && c.is_ascii_digit());
    let on_digits = keyboard.digits_layer.contains(c);
    let on_symbols = keyboard.symbols_layer.contains(c);
    // Stay on the current layer whenever the key exists there.
    match current {
        Layer::Letters if on_letters => Some(Layer::Letters),
        Layer::Digits if on_digits => Some(Layer::Digits),
        Layer::Symbols if on_symbols => Some(Layer::Symbols),
        _ if on_letters => Some(Layer::Letters),
        _ if on_digits => Some(Layer::Digits),
        _ if on_symbols => Some(Layer::Symbols),
        _ => None,
    }
}

fn distance(from: Layer, to: Layer) -> u32 {
    match (from, to) {
        (a, b) if a == b => 0,
        (Layer::Letters, Layer::Symbols) | (Layer::Symbols, Layer::Letters) => 2,
        _ => 1,
    }
}

impl Add for TypingCost {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            taps: self.taps + rhs.taps,
            characters: self.characters + rhs.characters,
            layer_switches: self.layer_switches + rhs.layer_switches,
            shift_taps: self.shift_taps + rhs.shift_taps,
            long_presses: self.long_presses + rhs.long_presses,
        }
    }
}

impl AddAssign for TypingCost {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

/// One counted interaction of a simulated session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapStep {
    /// What the person did, e.g. `type "abcde-12345" into code-input`.
    pub action: String,
    /// Taps it cost.
    pub taps: u32,
}

/// The running tap count of a simulated session.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TapLog {
    steps: Vec<TapStep>,
}

impl TapLog {
    /// Records a step.
    pub fn record(&mut self, action: impl Into<String>, taps: u32) {
        self.steps.push(TapStep { action: action.into(), taps });
    }

    /// All steps, oldest first.
    pub fn steps(&self) -> &[TapStep] {
        &self.steps
    }

    /// Total taps so far.
    pub fn total(&self) -> u32 {
        self.steps.iter().map(|s| s.taps).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kb() -> Keyboard {
        Keyboard::three_layer()
    }

    #[test]
    fn lower_case_letters_cost_one_tap_each() {
        let cost = TypingCost::of("ebook", &kb(), InputMode::Text);
        assert_eq!(cost, TypingCost { taps: 5, characters: 5, ..Default::default() });
        assert_eq!(TypingCost::of("", &kb(), InputMode::Text), TypingCost::default());
    }

    #[test]
    fn digits_need_a_layer_switch_that_sticks() {
        // a, [switch], 1, 2, 3
        let cost = TypingCost::of("a123", &kb(), InputMode::Text);
        assert_eq!(cost.taps, 5);
        assert_eq!(cost.layer_switches, 1);
        // a, [switch], 1, [switch back], b, [switch], 2
        let cost = TypingCost::of("a1b2", &kb(), InputMode::Text);
        assert_eq!(cost.taps, 7);
        assert_eq!(cost.layer_switches, 3);
    }

    #[test]
    fn a_v02_gift_code_needs_no_layer_switch_and_a_pair_code_is_half_the_work() {
        // docs/api.md v0.2: 12 consonants printed XXXX-XXXX-XXXX; input ignores case and
        // separators, so the cheapest way to type it is 12 plain letter keys.
        let compact = TypingCost::of("bcdfghjklmnp", &kb(), InputMode::Text);
        assert_eq!((compact.taps, compact.layer_switches, compact.shift_taps), (12, 0, 0));
        // Typed exactly as printed, the hyphens (digits layer) and capitals cost extra.
        let as_printed = TypingCost::of("BCDF-GHJK-LMNP", &kb(), InputMode::Text);
        assert!(as_printed.layer_switches >= 4, "{as_printed:?}");
        assert!(as_printed.taps > compact.taps + 4);
        // The pair code (XXX-XXX) is six letter keys.
        let pair = TypingCost::of("bcdfgh", &kb(), InputMode::Text);
        assert_eq!((pair.taps, pair.layer_switches), (6, 0));
        // The v0.1 code (5 letters + 5 digits) always paid at least one layer switch.
        assert_eq!(TypingCost::of("abcde12345", &kb(), InputMode::Text).layer_switches, 1);
    }

    #[test]
    fn a_printed_code_is_cheaper_in_lower_case_and_cheapest_without_hyphen() {
        let as_printed = TypingCost::of("ABCDE-12345", &kb(), InputMode::Text);
        // caps lock (2) + 5 letters + switch (1) + hyphen + 5 digits
        assert_eq!(as_printed.taps, 14);
        assert_eq!(as_printed.shift_taps, 2);
        let lower = TypingCost::of("abcde-12345", &kb(), InputMode::Text);
        assert_eq!(lower.taps, 12);
        let compact = TypingCost::of("abcde12345", &kb(), InputMode::Text);
        assert_eq!(compact.taps, 11);
        assert!(compact.taps < lower.taps && lower.taps < as_printed.taps);
    }

    #[test]
    fn interleaved_letters_and_digits_are_the_worst_case_for_codes() {
        let grouped = TypingCost::of("abcde12345", &kb(), InputMode::Text);
        let interleaved = TypingCost::of("a1b2c3d4e5", &kb(), InputMode::Text);
        assert_eq!(interleaved.layer_switches, 9);
        assert!(interleaved.taps > grouped.taps + 5);
        // A number row removes the penalty entirely.
        let row = TypingCost::of("a1b2c3d4e5", &Keyboard::with_number_row(), InputMode::Text);
        assert_eq!(row.taps, 10);
        assert_eq!(row.layer_switches, 0);
    }

    #[test]
    fn single_capitals_use_one_shot_shift_and_runs_use_caps_lock() {
        assert_eq!(TypingCost::of("Ab", &kb(), InputMode::Text).taps, 3);
        // Two capitals: one-shot twice (4) beats lock + unlock (2 + 1 + 2 = 5 with trailing lower-case).
        assert_eq!(TypingCost::of("ABc", &kb(), InputMode::Text).taps, 5);
        // Five capitals then lower-case: lock (2) + 5 + unlock (1) + 1.
        assert_eq!(TypingCost::of("ABCDEf", &kb(), InputMode::Text).taps, 9);
        let no_lock = Keyboard { caps_lock: false, ..kb() };
        assert_eq!(TypingCost::of("ABCDE", &no_lock, InputMode::Text).taps, 10);
    }

    #[test]
    fn urls_drop_the_scheme_and_get_the_slash_key() {
        assert_eq!(typed_url("https://EBK.example/"), "ebk.example");
        assert_eq!(typed_url("http://ebk.example/k/ABCDE12345"), "ebk.example/k/ABCDE12345");
        assert_eq!(typed_url("ebk.example"), "ebk.example");
        assert_eq!(typed_url("http://127.0.0.1:8080/e?x=1"), "127.0.0.1:8080/e?x=1");
        let cost = TypingCost::of_url("https://ebk.example/e", &kb());
        // "ebk.example/e": 13 characters, all on the URL letters layer.
        assert_eq!(cost, TypingCost { taps: 13, characters: 13, ..Default::default() });
        // In a text field the slash is on the digits layer: 2 switches more.
        assert_eq!(TypingCost::of("ebk.example/e", &kb(), InputMode::Text).taps, 15);
    }

    #[test]
    fn unusual_characters_cost_a_long_press_and_never_panic() {
        let cost = TypingCost::of("é", &kb(), InputMode::Text);
        assert_eq!(cost.taps, 2);
        assert_eq!(cost.long_presses, 1);
        let cost = TypingCost::of("1€", &kb(), InputMode::Text);
        // switch, 1, switch back, long-press + pick
        assert_eq!(cost.taps, 5);
        let weird = "\u{0}\u{202e}💥\n\tＡ";
        let cost = TypingCost::of(weird, &kb(), InputMode::Text);
        assert_eq!(cost.characters, 6);
        assert!(cost.taps >= 6);
    }

    #[test]
    fn symbols_are_two_layers_away_from_letters() {
        let cost = TypingCost::of("a#", &kb(), InputMode::Text);
        assert_eq!(cost.taps, 4);
        assert_eq!(cost.layer_switches, 2);
    }

    #[test]
    fn costs_add_up_and_the_log_totals_them() {
        let a = TypingCost::of("ab", &kb(), InputMode::Text);
        let b = TypingCost::of("12", &kb(), InputMode::Text);
        assert_eq!((a + b).taps, 5);
        let mut log = TapLog::default();
        log.record("open the browser", 2);
        log.record("type the address", (a + b).taps);
        assert_eq!(log.total(), 7);
        assert_eq!(log.steps().len(), 2);
    }
}
