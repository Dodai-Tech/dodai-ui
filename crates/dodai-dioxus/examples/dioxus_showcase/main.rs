//! The Dodai showcase in Dioxus: the same pages, sections and controls as the gpui showcase,
//! built from `dodai-showcase-model` with dodai-dioxus components.
//!
//! `cargo run -p dodai-dioxus --example dioxus_showcase` serves the live app (events run in this
//! process over a websocket) at <http://127.0.0.1:3030>; `DODAI_PORT` changes the port.
//! `-- --html out.html` renders the page once to a static file instead. `DODAI_THEME`,
//! `DODAI_CAT` and `DODAI_FILTER` choose what it opens on.

mod app;
mod render;

use axum::{response::Redirect, routing::get, Router};
use dioxus::prelude::*;
use dioxus_liveview::LiveviewRouter;

/// The page as a standalone HTML document (static: no events).
fn static_html() -> String {
    let mut dom = VirtualDom::new(app::App);
    dom.rebuild_in_place();
    let body = dioxus_ssr::render(&dom);
    format!(
        "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Dodai UI · Dioxus</title><link rel=\"icon\" href=\"{}\"></head><body style=\"margin:0\">{body}</body></html>",
        dodai_showcase_model::media::logo_data_url()
    )
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(at) = args.iter().position(|a| a == "--html") {
        let out = args
            .get(at + 1)
            .map_or("dioxus_showcase.html", String::as_str);
        std::fs::write(out, static_html()).expect("write the HTML file");
        println!("{out}");
        return;
    }
    let port: u16 = std::env::var("DODAI_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3030);
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();
    // The liveview route is a catch-all below the root (axum needs a non-empty path); `/` redirects to `/index`.
    let router = Router::new()
        .route("/", get(|| async { Redirect::to("/index") }))
        .with_app("/", app::App);
    println!("Listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind the port");
    axum::serve(listener, router.into_make_service())
        .await
        .expect("serve");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_static_page_has_the_chrome_and_a_section() {
        let html = static_html();
        for want in [
            "dodai-app",
            "dodai-side",
            "dodai-topbar",
            "dodai-page",
            "Filter components",
            "dodai-select",
            "Design",
            "Foundations",
        ] {
            assert!(html.contains(want), "missing {want}");
        }
        // Every category is in the sidebar and its count is shown.
        for c in dodai_showcase_model::CATEGORIES {
            assert!(html.contains(c), "missing category {c}");
        }
    }

    #[test]
    fn the_page_is_themed_through_the_provider() {
        let html = static_html();
        assert!(html.contains("data-theme="));
        assert!(html.contains("--dodai-kit-card-elevated-bg"));
    }
}
