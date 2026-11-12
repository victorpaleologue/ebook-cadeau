//! `{placeholder}` substitution for message templates.

/// Prefix of a placeholder that applies French elision: `{de:name}` renders as
/// `de Léo` or `d’Alice`.
const DE_PREFIX: &str = "de:";

/// Replaces `{name}` placeholders in `template` with the matching values of `args`.
///
/// * Substitution is **single-pass**: values are inserted verbatim and never scanned for
///   further placeholders, so a value such as `"{url}"` cannot pull in another argument.
/// * A placeholder with no matching argument is left untouched, braces included; an
///   unmatched `{` is copied as is. Arguments that match nothing are ignored. If a name
///   is given twice, the first value wins.
/// * `{de:name}` is the French *de* + value with elision: `de Léo`, `d’Alice`,
///   `d’Hélène`, `d’Yves`, `de Yann`.
/// * Nothing is escaped: the result is plain text.
///
/// ```
/// use ebc_i18n::format_template;
/// assert_eq!(format_template("Pour {name}", &[("name", "Léo")]), "Pour Léo");
/// assert_eq!(format_template("{a} {b}", &[("a", "{b}"), ("b", "x")]), "{b} x");
/// assert_eq!(format_template("Hello {who}", &[]), "Hello {who}");
/// assert_eq!(format_template("De la part {de:name}", &[("name", "Élise")]), "De la part d’Élise");
/// ```
#[must_use]
pub fn format_template(template: &str, args: &[(&str, &str)]) -> String {
    let lookup = |name: &str| args.iter().find(|(key, _)| *key == name).map(|(_, value)| *value);
    let mut out = String::with_capacity(template.len());
    for piece in Pieces::new(template) {
        match piece {
            Piece::Text(text) => out.push_str(text),
            Piece::Placeholder(name) => {
                let replaced = match name.strip_prefix(DE_PREFIX) {
                    Some(plain) => lookup(plain).map(french_de),
                    None => lookup(name).map(str::to_owned),
                };
                match replaced {
                    Some(value) => out.push_str(&value),
                    None => {
                        out.push('{');
                        out.push_str(name);
                        out.push('}');
                    }
                }
            }
        }
    }
    out
}

/// One piece of a template.
enum Piece<'a> {
    /// Literal text, copied as is.
    Text(&'a str),
    /// The name between a `{` and the next `}` (no `{` in between).
    Placeholder(&'a str),
}

/// Splits a template into literal text and placeholders in linear time: each step looks
/// for the next `}` and then for the last `{` before it, so a placeholder name never
/// contains a brace and unbalanced braces are plain text.
struct Pieces<'a> {
    rest: &'a str,
    pending: Option<&'a str>,
}

impl<'a> Pieces<'a> {
    fn new(template: &'a str) -> Self {
        Self { rest: template, pending: None }
    }
}

impl<'a> Iterator for Pieces<'a> {
    type Item = Piece<'a>;

    fn next(&mut self) -> Option<Piece<'a>> {
        if let Some(name) = self.pending.take() {
            return Some(Piece::Placeholder(name));
        }
        if self.rest.is_empty() {
            return None;
        }
        let Some(close) = self.rest.find('}') else {
            return Some(Piece::Text(std::mem::take(&mut self.rest)));
        };
        let (head, tail) = self.rest.split_at(close);
        let after = tail.get(1..).unwrap_or_default();
        match head.rfind('{') {
            Some(open) => {
                self.rest = after;
                self.pending = Some(head.get(open + 1..).unwrap_or_default());
                Some(Piece::Text(head.get(..open).unwrap_or_default()))
            }
            None => {
                // A closing brace without an opening one: literal text up to and
                // including it.
                let literal = self.rest.get(..=close).unwrap_or(self.rest);
                self.rest = after;
                Some(Piece::Text(literal))
            }
        }
    }
}

/// The placeholder names used in `template`, in order of first appearance, without
/// duplicates and without the `de:` elision prefix.
///
/// ```
/// use ebc_i18n::placeholders_in;
/// assert_eq!(placeholders_in("{a} et {b}, puis {a}"), ["a", "b"]);
/// assert_eq!(placeholders_in("De la part {de:name}"), ["name"]);
/// assert!(placeholders_in("no placeholder, lone { brace").is_empty());
/// ```
#[must_use]
pub fn placeholders_in(template: &str) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    for piece in Pieces::new(template) {
        if let Piece::Placeholder(name) = piece {
            let name = name.strip_prefix(DE_PREFIX).unwrap_or(name);
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

/// French *de* followed by `value`, elided to *d’* before a vowel sound.
///
/// The rule is the one that works for given names: elide before a vowel (accented or
/// not), before `h` (mute in almost every French given name: Hélène, Henri, Hugo), and
/// before `y` followed by a consonant (Yves, Yvonne) but not `y` followed by a vowel
/// (Yann, Yasmine).
fn french_de(value: &str) -> String {
    let mut chars = value.chars();
    let first = chars.next().map(fold_letter);
    let second = chars.next().map(fold_letter);
    let is_vowel = |c: char| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u');
    let elide = match (first, second) {
        (Some(c), _) if is_vowel(c) || c == 'h' => true,
        (Some('y'), Some(next)) => next.is_alphabetic() && !is_vowel(next) && next != 'y',
        _ => false,
    };
    if elide { format!("d\u{2019}{value}") } else { format!("de {value}") }
}

/// Lower-cases a letter and strips the diacritics that occur on French vowels.
fn fold_letter(c: char) -> char {
    match c.to_lowercase().next().unwrap_or(c) {
        'à' | 'â' | 'ä' | 'á' | 'ã' | 'å' | 'æ' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'î' | 'ï' | 'í' | 'ì' => 'i',
        'ô' | 'ö' | 'ó' | 'ò' | 'õ' | 'œ' | 'ø' => 'o',
        'ù' | 'û' | 'ü' | 'ú' => 'u',
        'ÿ' | 'ý' => 'y',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_known_placeholders_only() {
        assert_eq!(format_template("", &[("a", "x")]), "");
        assert_eq!(format_template("plain", &[("a", "x")]), "plain");
        assert_eq!(format_template("{a}", &[("a", "x")]), "x");
        assert_eq!(format_template("{a}{a} {b}", &[("a", "x"), ("b", "y")]), "xx y");
        assert_eq!(format_template("{a} {missing}", &[("a", "x")]), "x {missing}");
        assert_eq!(format_template("{a}", &[("b", "y")]), "{a}");
        assert_eq!(format_template("{a}", &[("a", "first"), ("a", "second")]), "first");
        assert_eq!(format_template("{a}", &[("a", "")]), "");
        assert_eq!(format_template("{}", &[("", "empty name")]), "empty name");
        assert_eq!(format_template("é{a}漢", &[("a", "ß")]), "éß漢");
    }

    #[test]
    fn substitution_is_single_pass() {
        // A value that looks like a placeholder is not expanded again.
        assert_eq!(format_template("{a} {b}", &[("a", "{b}"), ("b", "x")]), "{b} x");
        assert_eq!(format_template("{a}", &[("a", "{a}")]), "{a}");
        assert_eq!(format_template("{a}", &[("a", "{a}{a}{a}")]), "{a}{a}{a}");
        // A hostile name cannot inject the URL of another field.
        let out = format_template(
            "{name} vous offre ce livre. Rendez-vous sur {url}.",
            &[("name", "{url}"), ("url", "https://ebk.example")],
        );
        assert_eq!(out, "{url} vous offre ce livre. Rendez-vous sur https://ebk.example.");
    }

    #[test]
    fn malformed_braces_are_copied() {
        for template in
            ["{", "}", "{{", "}}", "{a", "a}", "{ {a}", "}{", "{a{b}", "{{a}}", "{\u{0}}"]
        {
            // No argument matches: the output is the input.
            assert_eq!(format_template(template, &[]), template, "{template:?}");
        }
        assert_eq!(format_template("{ {a}", &[("a", "x")]), "{ x");
        assert_eq!(format_template("{a{b}", &[("b", "x")]), "{ax");
        assert_eq!(format_template("{{a}}", &[("a", "x")]), "{x}");
        assert_eq!(format_template("{a", &[("a", "x")]), "{a");
        assert_eq!(format_template("100 % {a} }", &[("a", "x")]), "100 % x }");
    }

    #[test]
    fn large_inputs_are_handled() {
        let template = "{a}".repeat(50_000);
        let out = format_template(&template, &[("a", "xy")]);
        assert_eq!(out.len(), 100_000);
        let braces = "{".repeat(200_000);
        assert_eq!(format_template(&braces, &[("a", "x")]), braces);
        assert!(placeholders_in(&braces).is_empty());
        assert_eq!(placeholders_in(&template), ["a"]);
    }

    #[test]
    fn lists_placeholders() {
        assert!(placeholders_in("").is_empty());
        assert!(placeholders_in("no braces").is_empty());
        assert_eq!(placeholders_in("{url}"), ["url"]);
        assert_eq!(placeholders_in("{b} {a} {b}"), ["b", "a"]);
        assert_eq!(placeholders_in("{de:name} {name}"), ["name"]);
        assert_eq!(placeholders_in("{ {a}"), ["a"]);
        assert_eq!(placeholders_in("{}"), [""]);
        assert!(placeholders_in("{a").is_empty());
    }

    #[test]
    fn french_elision() {
        let de = |name: &str| format_template("{de:name}", &[("name", name)]);
        for (name, expected) in [
            ("Léo", "de Léo"),
            ("Marie", "de Marie"),
            ("Alice", "d’Alice"),
            ("alice", "d’alice"),
            ("Élise", "d’Élise"),
            ("Émile", "d’Émile"),
            ("Inès", "d’Inès"),
            ("Océane", "d’Océane"),
            ("Ulysse", "d’Ulysse"),
            ("Œdipe", "d’Œdipe"),
            ("Hélène", "d’Hélène"),
            ("Henri", "d’Henri"),
            ("Hugo", "d’Hugo"),
            ("Yves", "d’Yves"),
            ("Yvonne", "d’Yvonne"),
            ("Yann", "de Yann"),
            ("Yasmine", "de Yasmine"),
            ("Youssef", "de Youssef"),
            ("Y", "de Y"),
            ("Mamie Odette", "de Mamie Odette"),
            ("", "de "),
            ("42", "de 42"),
            ("{url}", "de {url}"),
            ("漢", "de 漢"),
        ] {
            assert_eq!(de(name), expected, "name {name:?}");
        }
        // Without an argument the placeholder stays, prefix included.
        assert_eq!(format_template("{de:name}", &[]), "{de:name}");
        // The plain placeholder and the eliding one can share an argument.
        assert_eq!(format_template("{name} / {de:name}", &[("name", "Anna")]), "Anna / d’Anna");
    }
}
