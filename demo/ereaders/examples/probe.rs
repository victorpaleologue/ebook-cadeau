//! Temporary timing probe.
#![allow(missing_docs, clippy::print_stdout)]
use ebc_demo_ereaders::Page;
fn main() {
    let mib = 1024 * 1024;
    let cases: Vec<(&str, String)> = vec![
        ("nested div", "<div>".repeat(mib / 5)),
        ("nested b", "<b>".repeat(mib / 3)),
        ("nested b id", (0..mib / 12).map(|i| format!("<b id={i}>")).collect()),
        ("nested table", "<table><tr><td>".repeat(mib / 15)),
        ("nested a", "<a href=x>".repeat(mib / 10)),
        ("nested form/label", "<label><form>".repeat(mib / 13)),
        ("unclosed p", "<p>x".repeat(mib / 4)),
        ("nested span style", "<span style='color:#ccc'>x".repeat(mib / 26)),
        ("nested select", "<select><option>".repeat(mib / 16)),
        ("nested li", "<ul><li>".repeat(mib / 8)),
        ("nested font", "<font color=red>x".repeat(mib / 17)),
        ("nested nobr", "<nobr>x".repeat(mib / 7)),
        ("nested button", "<button>x".repeat(mib / 9)),
        ("many attrs", format!("<p {}>", (0..mib / 8).map(|i| format!("a{i}=1 ")).collect::<String>())),
        ("misnested formatting", "<b><p>x</b>".repeat(mib / 11)),
        ("svg nest", "<svg><g>".repeat(mib / 8)),
    ];
    for (name, html) in cases {
        let t = std::time::Instant::now();
        let p = Page::from_html("http://x/", &html);
        let parse = t.elapsed();
        let t = std::time::Instant::now();
        let r = p.lint(&Default::default());
        println!("{name:24} {:8} bytes parse {:?} lint {:?} findings {}", html.len(), parse, t.elapsed(), r.findings.len());
    }
}
