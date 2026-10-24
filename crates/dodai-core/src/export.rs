//! Importable design tokens.
//!
//! [`design_tokens_json`] writes the themes as a [W3C Design Tokens (DTCG)]
//! document (`$type` / `$value`). Tools that read it: Style Dictionary v4,
//! Tokens Studio / Figma Variables importers, Penpot, Specify, and anything
//! else that speaks DTCG. No dependencies; the JSON is built by hand.
//!
//! [W3C Design Tokens (DTCG)]: https://tr.designtokens.org/format/

use crate::{Color, Theme};
use std::fmt::Write as _;

/// JSON string literal with escaping.
fn js(s: &str) -> String {
    let mut o = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            c if (c as u32) < 0x20 => {
                let _ = write!(o, "\\u{:04x}", c as u32);
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn num(v: f32) -> String {
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn color(c: Color) -> String {
    // 8-digit hex keeps alpha for translucent tokens.
    if c.a == 255 {
        c.to_hex()
    } else {
        format!("{}{:02x}", c.to_hex(), c.a)
    }
}

fn tok(ty: &str, value: String) -> String {
    format!("{{\"$type\":{},\"$value\":{}}}", js(ty), value)
}

fn group(name: &str, members: &[(String, String)]) -> String {
    let body: Vec<String> = members
        .iter()
        .map(|(k, v)| format!("{}:{}", js(k), v))
        .collect();
    format!("{}:{{{}}}", js(name), body.join(","))
}

fn px(v: f32) -> String {
    tok("dimension", js(&format!("{}px", num(v))))
}

/// The `color` group: palette, roles and the numbered series.
fn colour_tokens(t: &Theme) -> Vec<(String, String)> {
    let p = &t.palette;
    let c = |name: &str, col: Color| (name.to_string(), tok("color", js(&color(col))));
    let mut colors = vec![
        c("background", p.background),
        c("surface", p.surface),
        c("surface-raised", p.surface_raised),
        c("overlay", p.overlay),
        c("border", p.border),
        c("border-strong", p.border_strong),
        c("text", p.text),
        c("text-muted", p.text_muted),
        c("text-disabled", p.text_disabled),
        c("primary", p.primary),
        c("primary-hover", p.primary_hover),
        c("primary-active", p.primary_active),
        c("on-primary", p.on_primary),
        c("secondary", p.secondary),
        c("secondary-hover", p.secondary_hover),
        c("on-secondary", p.on_secondary),
        c("accent", p.accent),
        c("info", p.info),
        c("success", p.success),
        c("warning", p.warning),
        c("danger", p.danger),
        c("selection", p.selection),
        c("focus-ring", p.focus_ring),
        c("link", p.link),
    ];
    for (name, col) in p.roles.list() {
        colors.push(c(name, col));
    }
    let series: Vec<(String, String)> = p
        .series
        .iter()
        .enumerate()
        .map(|(i, s)| ((i + 1).to_string(), tok("color", js(&color(*s)))))
        .collect();
    colors.push((
        "series".into(),
        format!("{{{}}}", {
            let b: Vec<String> = series
                .iter()
                .map(|(k, v)| format!("{}:{}", js(k), v))
                .collect();
            b.join(",")
        }),
    ));
    colors
}

/// The `font` group.
fn font_tokens(t: &Theme) -> Vec<(String, String)> {
    let ty = &t.typography;
    let font = vec![
        (
            "family".to_string(),
            format!(
                "{{{},{},{}}}",
                format_args!("\"display\":{}", tok("fontFamily", js(ty.display))),
                format_args!("\"body\":{}", tok("fontFamily", js(ty.body))),
                format_args!("\"mono\":{}", tok("fontFamily", js(ty.mono))),
            ),
        ),
        (
            "size".to_string(),
            format!(
                "{{{}}}",
                [
                    ("xs", ty.size_xs),
                    ("sm", ty.size_sm),
                    ("md", ty.size_md),
                    ("lg", ty.size_lg),
                    ("xl", ty.size_xl),
                    ("xxl", ty.size_xxl),
                ]
                .iter()
                .map(|(k, v)| format!("\"{k}\":{}", px(*v)))
                .collect::<Vec<_>>()
                .join(",")
            ),
        ),
        (
            "label-tracking".to_string(),
            tok("number", num(ty.label_tracking)),
        ),
        (
            "label-case".to_string(),
            format!(
                "{{\"$type\":\"string\",\"$value\":{}}}",
                js(if ty.uppercase_labels {
                    "uppercase"
                } else {
                    "none"
                })
            ),
        ),
    ];
    font
}

/// The `space` group.
fn space_tokens(t: &Theme) -> Vec<(String, String)> {
    let s = &t.spacing;
    let space: Vec<(String, String)> = [
        ("xs", s.xs),
        ("sm", s.sm),
        ("md", s.md),
        ("lg", s.lg),
        ("xl", s.xl),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), px(*v)))
    .collect();
    space
}

/// The `shape` group.
fn shape_tokens(t: &Theme) -> Vec<(String, String)> {
    let sh = &t.shape;
    let corners = [
        sh.corners.top_left,
        sh.corners.top_right,
        sh.corners.bottom_right,
        sh.corners.bottom_left,
    ]
    .iter()
    .map(|b| if *b { "1" } else { "0" })
    .collect::<Vec<_>>()
    .join("");
    let shape = vec![
        ("radius".to_string(), px(sh.radius)),
        ("chamfer".to_string(), px(sh.chamfer)),
        ("border-width".to_string(), px(sh.border_width)),
        ("control-height".to_string(), px(sh.control_height)),
        (
            // clockwise from top-left, 1 = cut
            "chamfered-corners".to_string(),
            format!("{{\"$type\":\"string\",\"$value\":{}}}", js(&corners)),
        ),
    ];
    shape
}

/// The `effect` group.
fn effect_tokens(t: &Theme) -> Vec<(String, String)> {
    let e = &t.effects;
    let effect = vec![
        ("glow".to_string(), tok("color", js(&color(e.glow)))),
        ("glow-blur".to_string(), px(e.glow_blur)),
        (
            "glow-intensity".to_string(),
            tok("number", num(e.glow_intensity)),
        ),
        ("hard-shadow".to_string(), px(e.hard_shadow)),
        (
            "scanlines".to_string(),
            format!(
                "{{\"$type\":\"string\",\"$value\":{}}}",
                js(if e.scanlines { "on" } else { "off" })
            ),
        ),
    ];
    effect
}

/// The `motion` group.
fn motion_tokens(t: &Theme) -> Vec<(String, String)> {
    let m = &t.motion;
    let motion = vec![
        (
            "duration".to_string(),
            tok("duration", js(&format!("{}ms", m.duration.as_millis()))),
        ),
        (
            "delay".to_string(),
            tok("duration", js(&format!("{}ms", m.delay.as_millis()))),
        ),
        ("easing".to_string(), tok("string", js(&m.easing.to_css()))),
    ];
    motion
}

/// The `surface` group: textures and effect strengths.
fn surface_tokens(t: &Theme) -> Vec<(String, String)> {
    let s = &t.surface_style;
    let fx = &s.preset;
    let ef = &s.effects;
    let word = |v: String| tok("string", js(&v));
    let owned_effects: Vec<&str> = [
        (ef.glitch, "glitch"),
        (ef.sheen, "sheen"),
        (ef.flicker, "flicker"),
        (ef.bloom, "bloom"),
        (ef.pulse, "pulse"),
        (ef.chase, "chase"),
        (ef.edge, "edge"),
        (ef.brackets, "brackets"),
        (ef.depth, "depth"),
        (ef.neon, "neon"),
        (ef.ripple, "ripple"),
        (ef.tint, "tint"),
        (ef.pop, "pop"),
        (ef.tape, "tape"),
        (ef.gloss, "gloss"),
        (ef.strip, "strip"),
    ]
    .iter()
    .filter_map(|(on, n)| on.then_some(*n))
    .collect();
    let surface = vec![
        (
            "textures".to_string(),
            word(
                s.textures
                    .iter()
                    .filter(|x| **x != crate::Texture::None)
                    .map(|x| x.label())
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        ),
        ("effects".to_string(), word(owned_effects.join(","))),
        ("texture".to_string(), word(fx.texture.label().to_string())),
        (
            "texture-strength".to_string(),
            tok("number", num(fx.strength)),
        ),
        (
            "container-texture".to_string(),
            word(fx.container_texture.label().to_string()),
        ),
        (
            "container-strength".to_string(),
            tok("number", num(fx.container_strength)),
        ),
        ("sheen".to_string(), tok("number", num(fx.sheen))),
        ("glitch".to_string(), tok("number", num(fx.glitch))),
        ("flicker".to_string(), tok("number", num(fx.flicker))),
        ("bloom".to_string(), tok("number", num(fx.bloom))),
        ("pulse".to_string(), tok("number", num(fx.pulse))),
        ("chase".to_string(), tok("number", num(fx.chase))),
        ("edge".to_string(), tok("number", num(fx.edge))),
        ("brackets".to_string(), tok("number", num(fx.brackets))),
        ("depth".to_string(), tok("number", num(fx.depth))),
        ("ripple".to_string(), tok("number", num(fx.ripple))),
        ("tint".to_string(), tok("number", num(fx.tint))),
        ("pop".to_string(), tok("number", num(fx.pop))),
        ("tape".to_string(), tok("number", num(fx.tape))),
        ("gloss".to_string(), tok("number", num(fx.gloss))),
        ("strip".to_string(), tok("number", num(fx.strip))),
        ("clear".to_string(), tok("number", num(fx.clear))),
    ];
    surface
}

/// One theme as a DTCG group (`color`, `font`, `space`, `shape`, `effect`).
pub fn theme_tokens_json(t: &Theme) -> String {
    let (colors, font, space) = (colour_tokens(t), font_tokens(t), space_tokens(t));
    let (shape, effect) = (shape_tokens(t), effect_tokens(t));
    let (motion, surface) = (motion_tokens(t), surface_tokens(t));

    format!(
        "{}:{{\"$description\":{},\"$extensions\":{{\"dodai.mode\":{},\"dodai.family\":{}}},{},{},{},{},{},{},{}}}",
        js(t.id()),
        js(t.meta.description),
        js(if t.is_dark() { "dark" } else { "light" }),
        js(t.meta.family),
        group("color", &colors),
        group("font", &font),
        group("space", &space),
        group("shape", &shape),
        group("effect", &effect),
        group("motion", &motion),
        group("surface", &surface),
    )
}

/// All `themes` as one DTCG document, one top-level group per theme id.
pub fn design_tokens_json(themes: &[Theme]) -> String {
    let body: Vec<String> = themes.iter().map(theme_tokens_json).collect();
    format!("{{{}}}\n", body.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal structural JSON check: balanced, strings terminated, no trailing commas.
    fn well_formed(s: &str) -> bool {
        let (mut depth, mut in_str, mut esc) = (0i32, false, false);
        let mut prev = ' ';
        for ch in s.chars() {
            if in_str {
                if esc {
                    esc = false;
                } else if ch == '\\' {
                    esc = true;
                } else if ch == '"' {
                    in_str = false;
                }
            } else {
                match ch {
                    '"' => in_str = true,
                    '{' | '[' => depth += 1,
                    '}' | ']' => {
                        if prev == ',' {
                            return false;
                        }
                        depth -= 1;
                        if depth < 0 {
                            return false;
                        }
                    }
                    _ => {}
                }
                if !ch.is_whitespace() {
                    prev = ch;
                }
            }
        }
        depth == 0 && !in_str
    }

    #[test]
    fn every_builtin_theme_exports_well_formed_dtcg() {
        let themes = vec![crate::themes::neutral()];
        let j = design_tokens_json(&themes);
        assert!(well_formed(&j), "malformed JSON");
        for t in &themes {
            assert!(
                j.contains(&format!("\"{}\":{{", t.id())),
                "missing {}",
                t.id()
            );
        }
        for key in [
            "\"$type\":\"color\"",
            "\"$type\":\"fontFamily\"",
            "\"$type\":\"dimension\"",
            "\"series\"",
            "\"focus-ring\"",
        ] {
            assert!(j.contains(key), "missing {key}");
        }
    }

    #[test]
    fn values_match_the_theme() {
        let t = crate::themes::neutral();
        let j = theme_tokens_json(&t);
        assert!(j.contains(&format!("\"$value\":\"{}\"", t.palette.primary.to_hex())));
        assert!(j.contains(&format!("\"$value\":\"{}px\"", num(t.spacing.md))));
        assert!(j.contains(&format!(
            "\"dodai.mode\":\"{}\"",
            if t.is_dark() { "dark" } else { "light" }
        )));
    }

    #[test]
    fn escaping_and_numbers() {
        assert_eq!(js("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
        assert_eq!(num(1.0), "1");
        assert_eq!(num(0.125), "0.125");
        assert_eq!(num(12.5), "12.5");
    }

    #[test]
    fn translucent_colours_keep_alpha() {
        let c = Color::rgba(255, 0, 0, 128);
        assert_eq!(color(c).len(), 9);
        assert_eq!(color(Color::rgba(255, 0, 0, 255)), "#ff0000");
    }
}
