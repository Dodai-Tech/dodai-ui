//! Writes every importable artefact into a directory (default `dist/`):
//!
//! ```text
//! dist/tokens/dodai.tokens.json   W3C DTCG tokens, all themes
//! dist/tokens/<theme>.tokens.json one file per theme
//! dist/tokens/dodai-<system>.tokens.json  one design system only (cyberpunk, material)
//! dist/css/dodai.css              custom properties + component CSS, all themes
//! dist/css/dodai-<system>.css     the same, for one design system only
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
    // One bundle per design system, so a consumer can ship just Cyberpunk or just Material.
    for sys in themes::systems() {
        let system = sys.family;
        let own: Vec<_> = all
            .iter()
            .filter(|t| t.meta.family == system)
            .cloned()
            .collect();
        assert!(!own.is_empty(), "no themes for design system {system}");
        write(
            out.join(format!("tokens/dodai-{system}.tokens.json")),
            design_tokens_json(&own),
        );
        write(
            out.join(format!("css/dodai-{system}.css")),
            dodai_topcoat::multi_theme_stylesheet(&own),
        );
    }
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

    for &i in dodai_icons::cyberpunk::ALL {
        write(
            out.join(format!("icons/svg/{}.svg", i.name())),
            dodai_icons::cyberpunk::svg(i, 24, 2.0),
        );
    }
    write(
        out.join("icons/dodai-cyberpunk-icons.svg"),
        dodai_icons::cyberpunk::sprite(2.0),
    );
    let manifest: Vec<String> = dodai_icons::cyberpunk::ALL
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
        dodai_icons::cyberpunk::ALL.len(),
        out.display()
    );
}
