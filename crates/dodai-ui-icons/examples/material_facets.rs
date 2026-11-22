//! `cargo run -p dodai-ui-icons --features material --example material_facets > facets.html`
//! Smooth vs faceted, side by side, for design review.
use dodai_ui_icons::material::{outlined, MaterialSymbol};

fn path(polys: &[Vec<(f32, f32)>]) -> String {
    let mut d = String::new();
    for p in polys {
        for (i, (x, y)) in p.iter().enumerate() {
            d += &format!("{}{:.2} {:.2}", if i == 0 { 'M' } else { 'L' }, x, y);
        }
        d.push('Z');
    }
    d
}

fn main() {
    let names: Vec<String> = std::env::args().skip(1).collect();
    let names: Vec<&str> = if names.is_empty() {
        "home settings search favorite cloud notifications person lock mail map flight bolt palette language camera call chat wifi bluetooth pets sports_esports skull_ delete menu more_horiz star bookmark schedule shopping_cart build code terminal folder description sync battery_full volume_up mic public rocket_launch bug_report shield security".split(' ').collect()
    } else {
        names.iter().map(String::as_str).collect()
    };
    println!("<body style='background:#0b0b10;color:#999;font:11px monospace;margin:12px'><div style='display:grid;grid-template-columns:repeat(8,1fr);gap:6px'>");
    for n in names {
        let Some(i) = MaterialSymbol::ALL
            .iter()
            .find(|m| m.name() == n)
            .and_then(|&m| outlined::get(m))
        else {
            continue;
        };
        println!("<div style='border:1px solid #2a2a33;padding:6px;text-align:center'>");
        for polys in [i.polygons(), i.faceted()] {
            println!("<svg width='64' height='64' viewBox='0 0 24 24' fill='#fcee0a'><path fill-rule='nonzero' d='{}'/></svg>", path(&polys));
        }
        println!("<div>{n}</div></div>");
    }
    println!("</div></body>");
}
