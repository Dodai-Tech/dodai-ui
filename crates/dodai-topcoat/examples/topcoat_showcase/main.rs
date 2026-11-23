//! The Dodai showcase as server-rendered HTML: the same pages, sections and controls as the gpui
//! showcase, built from `dodai-showcase-model` with the `.dodai-*` components of `dodai-topcoat`.
//!
//! `cargo run -p dodai-topcoat --example topcoat_showcase` serves it at <http://127.0.0.1:3031>
//! (`DODAI_PORT` changes the port). The page is plain HTML and one script: a press, a change or
//! typing posts a one-line action (see `wire`) and the server answers with the page it now
//! draws. `-- --html out.html` writes the page once to a file instead. `DODAI_THEME`,
//! `DODAI_CAT` and `DODAI_FILTER` choose what it opens on.

mod chrome;
mod render;
mod shader;
mod wire;

use dodai_showcase_model::Showcase;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

const SCRIPT: &str = include_str!("app.js");

/// The page as a standalone HTML document.
fn document(m: &mut Showcase) -> String {
    let id = m.theme().id();
    format!(
        "<!DOCTYPE html><html data-theme=\"{id}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>Dodai UI · Topcoat</title><link rel=\"icon\" href=\"{}\"><style>html,body{{margin:0;height:100%}}body{{background:var(--dodai-background);color:var(--dodai-text);font-family:var(--dodai-font-body);font-size:var(--dodai-size-md);line-height:1.5}}</style></head><body>{}<script>{SCRIPT}</script><script>{}{}</script></body></html>",
        dodai_showcase_model::media::logo_data_url(),
        chrome::app(m),
        dodai_topcoat::TABS_SCRIPT,
        dodai_topcoat::MENU_SCRIPT
    )
}

/// One parsed request.
struct Request {
    method: String,
    path: String,
    query: String,
    body: String,
}

fn read_request(stream: &mut TcpStream) -> Option<Request> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?.to_string();
    let target = parts.next()?.to_string();
    let mut length = 0usize;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h).ok()? == 0 || h == "\r\n" || h == "\n" {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            if k.eq_ignore_ascii_case("content-length") {
                length = v.trim().parse().unwrap_or(0);
            }
        }
    }
    let mut body = vec![0u8; length];
    reader.read_exact(&mut body).ok()?;
    let (path, query) = target.split_once('?').unwrap_or((&target, ""));
    Some(Request {
        method,
        path: path.to_string(),
        query: query.to_string(),
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

fn respond(stream: &mut TcpStream, status: &str, mime: &str, body: &[u8]) {
    respond_with(stream, status, mime, body, "");
}

/// Percent-encode `text` for a header value.
fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// [`respond`] with one more header line (`""` for none).
fn respond_with(stream: &mut TcpStream, status: &str, mime: &str, body: &[u8], extra: &str) {
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n{extra}\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn handle(mut stream: TcpStream, model: &Mutex<Showcase>) {
    let Some(req) = read_request(&mut stream) else {
        return;
    };
    let html = "text/html; charset=utf-8";
    let lock = || {
        model
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    };
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/") => {
            let doc = document(&mut lock());
            respond(&mut stream, "200 OK", html, doc.as_bytes());
        }
        ("GET", "/view") => {
            let app = chrome::app(&mut lock());
            respond(&mut stream, "200 OK", html, app.as_bytes());
        }
        ("POST", "/act") => {
            let mut m = lock();
            wire::apply(&mut m, req.body.trim_end());
            // The page writes the clipboard: the browser owns it.
            let copy = match m.take_effect() {
                Some(dodai_showcase_model::icon_actions::Effect::Copy(t)) => {
                    format!("X-Copy: {}\r\n", encode(&t))
                }
                None => String::new(),
            };
            let out = if req.query.contains("part=side") {
                chrome::side(&m)
            } else {
                chrome::app(&mut m)
            };
            drop(m);
            respond_with(&mut stream, "200 OK", html, out.as_bytes(), &copy);
        }
        ("GET", "/shader") => match shader::png(&req.query) {
            Some(png) => respond(&mut stream, "200 OK", "image/png", &png),
            None => respond(&mut stream, "204 No Content", "text/plain", b""),
        },
        _ => respond(&mut stream, "404 Not Found", "text/plain", b"not found"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(at) = args.iter().position(|a| a == "--html") {
        let out = args
            .get(at + 1)
            .map_or("topcoat_showcase.html", String::as_str);
        std::fs::write(out, document(&mut Showcase::from_env())).expect("write the HTML file");
        println!("{out}");
        return;
    }
    let port: u16 = std::env::var("DODAI_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3031);
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("bind the port");
    println!("Listening on http://127.0.0.1:{port}");
    let model = Arc::new(Mutex::new(Showcase::from_env()));
    for stream in listener.incoming().flatten() {
        let model = Arc::clone(&model);
        std::thread::spawn(move || handle(stream, &model));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_has_the_chrome_the_script_and_a_section() {
        let html = document(&mut Showcase::default());
        for want in [
            "dodai-app",
            "dodai-side",
            "dodai-topbar",
            "dodai-page",
            "Filter components",
            "dodai-select",
            "Design",
            "data-theme=",
            "--dodai-kit-card-elevated-bg",
            "<script>",
        ] {
            assert!(html.contains(want), "missing {want}");
        }
        for c in dodai_showcase_model::CATEGORIES {
            assert!(html.contains(c), "missing category {c}");
        }
    }

    #[test]
    fn every_page_renders_without_panicking() {
        let mut m = Showcase::default();
        for c in 0..dodai_showcase_model::CATEGORIES.len() {
            m.update(dodai_showcase_model::Action::Category(Some(c)));
            let html = chrome::app(&mut m);
            assert!(html.contains("dodai-page"), "page {c}");
        }
    }

    #[test]
    fn a_pick_in_the_sidebar_asks_for_a_jump_once() {
        let mut m = Showcase::default();
        assert!(wire::apply(&mut m, "C|0"));
        let title = m.visible()[2].title;
        assert!(wire::apply(&mut m, &format!("S|{title}")));
        assert!(chrome::app(&mut m).contains(&format!("data-jump=\"{title}\"")));
        assert!(!chrome::app(&mut m).contains("data-jump"));
    }
}
