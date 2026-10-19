//! Writes every importable artefact into a directory (default `dist/`):
//!
//! ```text
//! dist/tokens/dodai.tokens.json   W3C DTCG tokens, all themes
//! dist/tokens/<theme>.tokens.json one file per theme
//! dist/css/dodai.css              custom properties + component CSS, all themes
//! dist/icons/svg/<name>.svg       one SVG per icon (currentColor)
//! dist/icons/dodai-cyberpunk-icons.svg      sprite sheet (<symbol id="dodai-<name>">)
//! dist/icons/manifest.json        name -> group
//! ```
//!
//! `cargo run -p dodai-topcoat --example export -- dist`

use dodai::themes;
use dodai_core::export::{design_tokens_json, theme_tokens_json};
use std::{fs, path::PathBuf};

fn write(path: PathBuf, body: String) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
}

fn main() {
    let out = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "dist".into()));
    let all = themes::all();

    write(
        out.join("tokens/dodai.tokens.json"),
        design_tokens_json(&all),
    );
    for t in &all {
        write(
            out.join(format!("tokens/{}.tokens.json", t.id())),
            format!("{{{}}}\n", theme_tokens_json(t)),
        );
    }
    write(
        out.join("css/dodai.css"),
        dodai_topcoat::multi_theme_stylesheet(&all),
    );

    for &i in dodai_cyberpunk_icons::ALL {
        write(
            out.join(format!("icons/svg/{}.svg", i.name())),
            dodai_cyberpunk_icons::svg(i, 24, 2.0),
        );
    }
    write(
        out.join("icons/dodai-cyberpunk-icons.svg"),
        dodai_cyberpunk_icons::sprite(2.0),
    );
    let manifest: Vec<String> = dodai_cyberpunk_icons::ALL
        .iter()
        .map(|i| format!("  \"{}\": \"{}\"", i.name(), i.group()))
        .collect();
    write(
        out.join("icons/manifest.json"),
        format!("{{\n{}\n}}\n", manifest.join(",\n")),
    );
    println!(
        "wrote {} themes and {} icons to {}",
        all.len(),
        dodai_cyberpunk_icons::ALL.len(),
        out.display()
    );
}
