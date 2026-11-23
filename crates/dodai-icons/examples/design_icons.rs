//! `cargo run -p dodai-icons --example design_icons -- dist/icons-design`
//!
//! Every icon of every design's pack as a 24 px SVG, for the design file (`just design`): one JSON
//! file per pack with `{ id, svg, filled? }` rows, and `symbols.json` with the dashboard symbols
//! per group. Colour is `currentColor`-free: the design file paints the nodes with its variables.

use dodai_icons::{glyph, symbol::Symbol, IconId, Pack, StrokeStyle};
use std::fmt::Write;

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "dist/icons-design".into());
    std::fs::create_dir_all(&out).expect("create output dir");
    for (pack, name) in [
        (Pack::Stroked, "stroked"),
        (Pack::Material, "material"),
        (Pack::Bold, "bold"),
    ] {
        let mut json = String::from("[");
        for (n, id) in IconId::ALL.iter().enumerate() {
            let Some(g) = glyph(*id, pack, false) else { continue };
            if n > 0 {
                json.push(',');
            }
            let svg = g.svg(24, "#000000");
            write!(json, "{{\"id\":\"{}\",\"svg\":\"{}\"}}", id.name(), esc(&svg)).unwrap();
        }
        json.push(']');
        std::fs::write(format!("{out}/{name}.json"), json).expect("write pack");
    }
    let mut json = String::from("[");
    for (n, s) in Symbol::ALL.iter().enumerate() {
        if n > 0 {
            json.push(',');
        }
        let svg = s.glyph_styled(StrokeStyle::ROUND).svg(24, "#000000");
        write!(
            json,
            "{{\"id\":\"{}\",\"group\":\"{}\",\"svg\":\"{}\"}}",
            s.name(),
            s.group().name(),
            esc(&svg)
        )
        .unwrap();
    }
    json.push(']');
    std::fs::write(format!("{out}/symbols.json"), json).expect("write symbols");
}
