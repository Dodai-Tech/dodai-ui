//! `cargo run -p dodai-cyberpunk-icons --example sheet > sheet.html`
//! Contact sheet of every icon, large with the 24-grid, for design review.

use dodai_cyberpunk_icons::{lookup, svg, IconName, Modifier, ALL};

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
    // Derived variants of a few bases (pass `variants` to show only these).
    for base in [
        "heart", "cloud", "camera", "bell", "mail", "folder", "user", "home",
    ] {
        for m in Modifier::ALL {
            let name = format!("{base}-{}", m.suffix());
            if IconName::from_name(&name).is_some() {
                continue;
            }
            let Some(shapes) = lookup(&name) else {
                continue;
            };
            let d: String = shapes
                .iter()
                .map(|(p, closed)| {
                    let pts: Vec<String> = p.iter().map(|(x, y)| format!("{x},{y}")).collect();
                    format!("M{}{}", pts.join("L"), if *closed { "Z" } else { "" })
                })
                .collect();
            println!(
                "<div style='border:1px solid #2a2a33;padding:6px;text-align:center'><svg width='72' height='72' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='1.75' stroke-linecap='square'><path d='{d}'/></svg><div style='color:#999'>{name}</div></div>"
            );
        }
    }
    println!("</div></body>");
}
