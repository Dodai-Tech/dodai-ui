//! `cargo run -p dodai-ui-icons --example symbol_sheet [group] [round|square|bold] [small] > sheet.svg`
//! One SVG contact sheet of the symbols (optionally one group), with names, for design review.

use dodai_ui_icons::symbol::Symbol;
use dodai_ui_icons::{Glyph, Points, StrokeStyle};

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
    // `small`: 20 px icons, no labels or frames - the real size they ship at.
    let small = std::env::args().nth(3).as_deref() == Some("small");
    let (cell, icon) = if small { (28.0, 20.0) } else { (96.0, 60.0) };
    let group = std::env::args().nth(1).filter(|g| g != "all");
    let style = match std::env::args().nth(2).as_deref() {
        Some("square") => StrokeStyle::SQUARE,
        Some("bold") => StrokeStyle::BOLD,
        _ => StrokeStyle::ROUND,
    };
    let (cap, join) = if style.round {
        ("round", "miter")
    } else {
        ("square", "miter")
    };
    let join = if style.round { "round" } else { join };
    let list: Vec<Symbol> = Symbol::ALL
        .iter()
        .copied()
        .filter(|s| group.as_deref().is_none_or(|g| s.group().name() == g))
        .collect();
    // Square sheet, so previews that crop to a square show everything.
    let cols = (list.len() as f32).sqrt().ceil() as usize;
    let rows = list.len().div_ceil(cols);
    let (w, h) = (cols as f32 * cell, rows as f32 * cell);
    println!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='{w}' height='{h}' viewBox='0 0 {w} {h}'>"
    );
    println!("<rect width='{w}' height='{h}' fill='#101014'/>");
    for (n, s) in list.iter().enumerate() {
        let (x, y) = ((n % cols) as f32 * cell, (n / cols) as f32 * cell);
        let Glyph::Lines(lines, _) = s.glyph_styled(style) else {
            continue;
        };
        let k = icon / 24.0;
        let ox = x + (cell - icon) / 2.0;
        let oy = y + if small { (cell - icon) / 2.0 } else { 8.0 };
        if !small {
            println!(
                "<rect x='{ox}' y='{oy}' width='{icon}' height='{icon}' fill='none' stroke='#2a2a33'/>"
            );
        }
        println!(
            "<path transform='translate({ox} {}) scale({k})' d='{}' fill='none' stroke='#e8e8f0' \
             stroke-width='{}' stroke-linecap='{cap}' stroke-linejoin='{join}'/>",
            oy,
            path(&lines),
            style.weight
        );
        let solid: String = style
            .solids(&lines)
            .iter()
            .map(|p| path(&[(p.clone(), true)]))
            .collect();
        if !solid.is_empty() {
            println!(
                "<path transform='translate({ox} {oy}) scale({k})' d='{solid}' fill='#e8e8f0'/>"
            );
        }
        if small {
            continue;
        }
        println!(
            "<text x='{}' y='{}' fill='#9a9aa8' font-family='monospace' font-size='10' text-anchor='middle'>{}</text>",
            x + cell / 2.0,
            y + cell - 12.0,
            s.name()
        );
    }
    println!("</svg>");
}
