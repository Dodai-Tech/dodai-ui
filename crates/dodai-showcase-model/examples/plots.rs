//! Writes every chart of the Charts page as drawn shapes, per design system, for the design file:
//! `cargo run -p dodai-showcase-model --example plots -- dist/plots`.
//!
//! The shapes are the showcase model's own plots (polygons, polylines, dots in pixel space), so
//! the design file draws exactly what every toolkit paints.

use std::fmt::Write as _;
use std::path::PathBuf;

use dodai_core::{themes, Color, Theme};
use dodai_showcase_model::catalog::{sections, Ctx};
use dodai_showcase_model::node::Node;
use dodai_showcase_model::plot::Plot;
use dodai_showcase_model::state::Values;

fn rgba(c: Color) -> String {
    format!("\"{}{:02x}\"", c.to_hex(), c.a)
}

fn pts(p: &[(f32, f32)]) -> String {
    let v: Vec<String> = p.iter().map(|(x, y)| format!("[{x:.1},{y:.1}]")).collect();
    format!("[{}]", v.join(","))
}

fn plot_json(title: &str, section: &str, p: &Plot) -> String {
    let fills: Vec<String> = p
        .fills
        .iter()
        .map(|f| {
            format!(
                "{{\"color\":{},\"points\":{}}}",
                rgba(f.color),
                pts(&f.points)
            )
        })
        .collect();
    let lines: Vec<String> = p
        .lines
        .iter()
        .map(|l| {
            format!(
                "{{\"color\":{},\"width\":{:.1},\"points\":{}}}",
                rgba(l.color),
                l.width,
                pts(&l.points)
            )
        })
        .collect();
    format!(
        "{{\"title\":{title:?},\"section\":{section:?},\"w\":{:.0},\"h\":{:.0},\"fills\":[{}],\"lines\":[{}]}}",
        p.w,
        p.h,
        fills.join(","),
        lines.join(",")
    )
}

/// Plots in tree order, each titled by the text that follows it in its tile.
fn collect(n: &Node, section: &str, out: &mut Vec<String>) {
    match n {
        Node::Stack(s) => {
            let mut i = 0;
            while i < s.items.len() {
                if let Node::Plot(p) = &s.items[i] {
                    let title = match s.items.get(i + 1) {
                        Some(Node::Text(t, _)) => t.clone(),
                        _ => String::new(),
                    };
                    out.push(plot_json(&title, section, p));
                } else {
                    collect(&s.items[i], section, out);
                }
                i += 1;
            }
        }
        Node::Scroll { body, .. } => collect(body, section, out),
        _ => {}
    }
}

fn default_theme(family: &str, all: Vec<Theme>) -> Theme {
    let pick = all.iter().position(|t| t.meta.id == family).unwrap_or(0);
    all.into_iter()
        .nth(pick)
        .expect("a design system has a theme")
}

fn main() {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "dist/plots".into()),
    );
    std::fs::create_dir_all(&out).expect("create output dir");
    let values = Values::default();
    for sys in themes::systems() {
        let theme = default_theme(sys.family, (sys.themes)());
        let ctx = Ctx {
            theme: &theme,
            values: &values,
        };
        let mut plots = Vec::new();
        for s in sections()
            .iter()
            .filter(|s| s.category == "Charts" && (s.shown)(&theme))
        {
            collect(&(s.build)(&ctx), s.title, &mut plots);
        }
        let mut text = String::new();
        let _ = write!(text, "[\n{}\n]\n", plots.join(",\n"));
        std::fs::write(out.join(format!("{}.json", sys.family)), text).expect("write plots");
        println!("{}: {} charts", sys.family, plots.len());
    }
}
