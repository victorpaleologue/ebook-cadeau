//! Helpers shared by the command-line tests: the binary, own-made images, fixture
//! directories and a tiny HTTP server on the loopback interface.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use assert_cmd::Command;

/// *L'Étranger*, Folio: the ISBN of the fixtures.
pub(crate) const ISBN: &str = "9782070360024";
/// Address of the invented redemption service.
pub(crate) const BASE_URL: &str = "https://ebk.example";
/// 2026-11-06 12:00:00 UTC, so that PDFs are reproducible.
pub(crate) const CREATED: &str = "1794000000";

/// The `ebc` binary, with the environment variables it reads cleared.
pub(crate) fn ebc() -> Command {
    let mut command = Command::cargo_bin("ebc").expect("the ebc binary is built");
    command.env_remove("EBC_GOOGLE_BOOKS_KEY").env_remove("EBC_CONTACT").env_remove("SOURCE_DATE_EPOCH");
    command
}

/// A well-formed gift code, grouped as printed.
pub(crate) fn code() -> String {
    ebc_model::GiftCode::from_payload(123_456_789_012).expect("in range").grouped()
}

/// `tests/fixtures/<path>`.
pub(crate) fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join(path)
}

/// The bytes of `tests/fixtures/<path>`.
pub(crate) fn fixture_bytes(path: &str) -> Vec<u8> {
    std::fs::read(fixture(path)).expect("fixture exists")
}

/// Standard output of a finished command, as text.
pub(crate) fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// Standard error of a finished command, as text.
pub(crate) fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The exit status of a finished command.
pub(crate) fn status(output: &std::process::Output) -> i32 {
    output.status.code().expect("the process exited by itself")
}

/// An own-made "cover": a diagonal colour gradient with deterministic noise and a few
/// flat blocks, as RGB bytes. Busy enough not to look like a placeholder tile.
pub(crate) fn art_rgb(width: u32, height: u32) -> Vec<u8> {
    let mut seed = 0x2545_F491_4F6C_DD1D_u64;
    let mut out = Vec::with_capacity((width * height * 3) as usize);
    for y in 0..height {
        for x in 0..width {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let noise = (seed & 0x3F) as u32;
            let block = ((x / (width / 6).max(1)) + (y / (height / 9).max(1))) % 3;
            let r = (x * 200 / width + noise + block * 20).min(255);
            let g = (y * 180 / height + noise + (2 - block) * 30).min(255);
            let b = ((x + y) * 150 / (width + height) + noise + 40).min(255);
            out.extend_from_slice(&[r as u8, g as u8, b as u8]);
        }
    }
    out
}

/// The own-made cover as a JPEG file.
pub(crate) fn cover_jpeg(width: u32, height: u32) -> Vec<u8> {
    let mut out = Vec::new();
    jpeg_encoder::Encoder::new(&mut out, 85)
        .encode(
            &art_rgb(width, height),
            u16::try_from(width).expect("fits"),
            u16::try_from(height).expect("fits"),
            jpeg_encoder::ColorType::Rgb,
        )
        .expect("JPEG encoding");
    out
}

/// A PNG file of one flat colour.
pub(crate) fn flat_png(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
    let image = image::RgbImage::from_pixel(width, height, image::Rgb(rgb));
    let mut out = std::io::Cursor::new(Vec::new());
    image.write_to(&mut out, image::ImageFormat::Png).expect("PNG encoding");
    out.into_inner()
}

/// One entry of a fixture index: how to match, and the answer.
pub(crate) struct Answer {
    /// `"url"` (exact) or `"url_contains"`.
    pub(crate) key: &'static str,
    /// The URL or the part of it.
    pub(crate) pattern: String,
    /// HTTP status.
    pub(crate) status: u16,
    /// `Content-Type`.
    pub(crate) content_type: &'static str,
    /// The body.
    pub(crate) body: Vec<u8>,
}

impl Answer {
    pub(crate) fn exact(url: &str, status: u16, content_type: &'static str, body: &[u8]) -> Self {
        Self { key: "url", pattern: url.to_owned(), status, content_type, body: body.to_vec() }
    }

    pub(crate) fn containing(
        part: &str,
        status: u16,
        content_type: &'static str,
        body: &[u8],
    ) -> Self {
        Self {
            key: "url_contains",
            pattern: part.to_owned(),
            status,
            content_type,
            body: body.to_vec(),
        }
    }
}

/// Writes a fixture directory (`index.json` plus one body file per answer) into `dir`.
pub(crate) fn write_fixtures(dir: &Path, answers: &[Answer]) {
    let mut index = Vec::new();
    for (number, answer) in answers.iter().enumerate() {
        let file = format!("body-{number}.bin");
        std::fs::write(dir.join(&file), &answer.body).expect("fixture body written");
        index.push(serde_json::json!({
            answer.key: answer.pattern,
            "status": answer.status,
            "content_type": answer.content_type,
            "file": file,
        }));
    }
    std::fs::write(dir.join("index.json"), serde_json::to_vec_pretty(&index).expect("JSON"))
        .expect("fixture index written");
}

/// A canned HTTP answer of the loopback server: status and body.
pub(crate) type Canned = (u16, Vec<u8>);

/// A minimal HTTP/1.1 server on the loopback interface: answers by the first route whose
/// pattern occurs in the request target, 404 otherwise, and records every request head.
pub(crate) struct Server {
    /// `http://127.0.0.1:<port>`.
    pub(crate) base: String,
    heads: Arc<Mutex<Vec<String>>>,
}

impl Server {
    pub(crate) fn start(routes: Vec<(&'static str, Canned)>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        let base = format!("http://{}", listener.local_addr().expect("address"));
        let heads = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&heads);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut head = Vec::new();
                let mut buffer = [0_u8; 1024];
                while !head.windows(4).any(|w| w == b"\r\n\r\n") {
                    match stream.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => head.extend_from_slice(&buffer[..n]),
                    }
                }
                let head = String::from_utf8_lossy(&head).into_owned();
                let target = head.split_whitespace().nth(1).unwrap_or("").to_owned();
                seen.lock().expect("lock").push(head);
                let (status, body) = routes
                    .iter()
                    .find(|(pattern, _)| target.contains(pattern))
                    .map_or((404, b"not found".to_vec()), |(_, answer)| answer.clone());
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&body);
            }
        });
        Self { base, heads }
    }

    /// The request heads received so far.
    pub(crate) fn heads(&self) -> Vec<String> {
        self.heads.lock().expect("lock").clone()
    }
}

/// A loopback address nothing listens on.
pub(crate) fn closed_port() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let address = listener.local_addr().expect("address");
    drop(listener);
    format!("http://{address}")
}
