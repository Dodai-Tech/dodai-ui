//! `cargo run -p dodai-icons --example symbol_sheet [group] > sheet.svg`
//! One SVG contact sheet of the symbols (optionally one group), with names, for design review.

use dodai_icons::symbol::Symbol;
use dodai_icons::{Glyph, Points};

const CELL: f32 = 96.0;
const ICON: f32 = 60.0;

fn path(lines: &[(Points, bool)]) -> String {
    lines
        .iter()
        .map(|(p, closed)| {
            let pts: Vec<String> = p.iter().map(|(x, y)| format!("{x:.2},{y:.2}")).collect();
            format!("M{}{}", pts.join("L"), if *closed { "Z" } else { "" })
        })
        .collect()
}

fn main() {
    let group = std::env::args().nth(1);
    let list: Vec<Symbol> = Symbol::ALL
        .iter()
        .copied()
        .filter(|s| group.as_deref().is_none_or(|g| s.group().name() == g))
        .collect();
    // Square sheet, so previews that crop to a square show everything.
    let cols = (list.len() as f32).sqrt().ceil() as usize;
    let rows = list.len().div_ceil(cols);
    let (w, h) = (cols as f32 * CELL, rows as f32 * CELL);
    println!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' viewBox='0 0 {w} {h}'>"
    );
    println!("<rect width='{w}' height='{h}' fill='#101014'/>");
    for (n, s) in list.iter().enumerate() {
        let (x, y) = ((n % cols) as f32 * CELL, (n / cols) as f32 * CELL);
        let Glyph::Round(lines) = s.glyph() else {
            continue;
        };
        let k = ICON / 24.0;
        let ox = x + (CELL - ICON) / 2.0;
        println!(
            "<rect x='{ox}' y='{}' width='{ICON}' height='{ICON}' fill='none' stroke='#2a2a33'/>",
            y + 8.0
        );
        println!(
            "<path transform='translate({ox} {}) scale({k})' d='{}' fill='none' stroke='#e8e8f0' \
             stroke-width='1.4' stroke-linecap='round' stroke-linejoin='round'/>",
            y + 8.0,
            path(&lines)
        );
        println!(
            "<text x='{}' y='{}' fill='#9a9aa8' font-family='monospace' font-size='10' text-anchor='middle'>{}</text>",
            x + CELL / 2.0,
            y + CELL - 12.0,
            s.name()
        );
    }
    println!("</svg>");
}
