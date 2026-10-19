//! `cargo run -p dodai-cyberpunk-icons --example sheet > sheet.html`
//! Contact sheet of every icon, large with the 24-grid, for design review.

use dodai_cyberpunk_icons::{svg, ALL};

fn main() {
    println!("<body style='background:#0b0b10;color:#fcee0a;font:11px monospace;margin:12px'>");
    println!("<div style='display:grid;grid-template-columns:repeat(12,1fr);gap:6px'>");
    for &i in ALL {
        println!(
            "<div style='border:1px solid #2a2a33;padding:6px;text-align:center'>{}<div style='color:#999'>{}</div></div>",
            svg(i, 72, 1.75),
            i.name()
        );
    }
    println!("</div></body>");
}
