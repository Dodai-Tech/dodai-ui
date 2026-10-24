//! # dodai-topcoat
//!
//! Generates the Dodai UI **stylesheet** for server-rendered web UIs
//! (built for [Topcoat](https://crates.io/crates/topcoat), works with any
//! framework): CSS custom properties for every token, a themed component layer
//! (`.dodai-btn`, `.dodai-panel`, `.dodai-input`, `.dodai-tag`, `.dodai-tabs`, `.dodai-table`, …
//! with chamfered cut-corners via `clip-path`) and a tiny client-side switcher.
//!
//! ```
//! use dodai::themes;
//! let css = dodai_topcoat::multi_theme_stylesheet(&themes::all());
//! assert!(css.contains("[data-theme=\"cyberpunk-cyan\"]"));
//! ```
//!
//! In a Topcoat page, inline it (it never contains `</style`):
//! `<style>(css)</style>` and call `<script>(dodai_topcoat::switcher_script(..))</script>`.

use dodai_core::{Shape, ShapeKind, Theme};
use std::fmt::Write;

/// Component layer; only references `var(--dodai-*)`.
pub const COMPONENTS_CSS: &str = include_str!("components.css");

/// Google Fonts stylesheet URL for the fallback display face (Rajdhani) and mono.
pub const GOOGLE_FONTS_URL: &str =
    "https://fonts.googleapis.com/css2?family=Rajdhani:wght@400;500;600;700&family=JetBrains+Mono:wght@400;600&display=swap";

const PROBE: f32 = 4096.0;

/// Symbolic CSS coordinate for a value measured on a `PROBE`-sized box.
fn coord(v: f32, c: f32) -> String {
    let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
    if near(v, 0.0) {
        "0".into()
    } else if near(v, PROBE) {
        "100%".into()
    } else if near(v, c) {
        format!("{c}px")
    } else {
        format!("calc(100% - {c}px)")
    }
}

/// Points of `shape`'s outline as CSS `x y` pairs (percent + px).
fn clip_points(shape: &Shape) -> Vec<String> {
    let c = if shape.kind() == ShapeKind::Chamfered {
        shape.chamfer
    } else {
        0.0
    };
    shape
        .outline(PROBE, PROBE)
        .into_iter()
        .map(|(x, y)| format!("{} {}", coord(x, c), coord(y, c)))
        .collect()
}

/// `clip-path` value for `shape`: `polygon(...)` generated from
/// [`Shape::outline`] when chamfered, `none` for rounded/square shapes.
pub fn clip_path(shape: &Shape) -> String {
    if shape.kind() == ShapeKind::Chamfered {
        format!("polygon({})", clip_points(shape).join(","))
    } else {
        "none".into()
    }
}

fn with_chamfer(shape: &Shape, chamfer: f32) -> Shape {
    Shape {
        chamfer: chamfer.max(0.0),
        scoop: false,
        ..shape.clone()
    }
}

/// Clip for the inner (fill) layer inset by the border width.
fn clip_inner(shape: &Shape) -> String {
    let k = 2.0 - std::f32::consts::SQRT_2;
    clip_path(&with_chamfer(shape, shape.chamfer - k * shape.border_width))
}

/// Hollow border ring: the outer outline plus the inset outline (`evenodd`).
/// Only chamfered themes need it (others draw the ring with an inset shadow);
/// otherwise a degenerate polygon that paints nothing.
fn clip_ring(shape: &Shape) -> String {
    if shape.kind() != ShapeKind::Chamfered {
        return "polygon(0 0,0 0,0 0)".into();
    }
    let bw = shape.border_width;
    let k = 2.0 - std::f32::consts::SQRT_2;
    let inner = with_chamfer(shape, shape.chamfer - k * bw);
    let c = inner.chamfer;
    let pts = inner.outline(PROBE, PROBE).into_iter().map(|(x, y)| {
        let f = |v: f32| {
            let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
            if near(v, 0.0) {
                format!("{bw}px")
            } else if near(v, PROBE) {
                format!("calc(100% - {bw}px)")
            } else if near(v, c) {
                format!("{}px", c + bw)
            } else {
                format!("calc(100% - {}px)", c + bw)
            }
        };
        format!("{} {}", f(x), f(y))
    });
    let mut all = clip_points(shape);
    all.extend(pts);
    format!("polygon(evenodd,{})", all.join(","))
}

/// Stylesheet for one theme (alias of [`stylesheet`]).
pub fn css(t: &Theme) -> String {
    stylesheet(t)
}

/// `--dodai-*` custom property declarations (no selector) for one theme.
pub fn theme_vars(t: &Theme) -> String {
    let p = &t.palette;
    let ty = &t.typography;
    let mut s = String::new();
    let mut v = |k: &str, val: String| {
        let _ = write!(s, "--dodai-{k}:{val};");
    };
    for (k, c) in [
        ("background", p.background),
        ("surface", p.surface),
        ("surface-raised", p.surface_raised),
        ("overlay", p.overlay),
        ("border", p.border),
        ("border-strong", p.border_strong),
        ("text", p.text),
        ("text-muted", p.text_muted),
        ("text-disabled", p.text_disabled),
        ("primary", p.primary),
        ("primary-hover", p.primary_hover),
        ("primary-active", p.primary_active),
        ("on-primary", p.on_primary),
        ("secondary", p.secondary),
        ("secondary-hover", p.secondary_hover),
        ("on-secondary", p.on_secondary),
        ("accent", p.accent),
        ("info", p.info),
        ("success", p.success),
        ("warning", p.warning),
        ("danger", p.danger),
        ("on-danger", t.on(p.danger)),
        ("selection", p.selection),
        ("focus-ring", p.focus_ring),
        ("link", p.link),
        ("glow", t.effects.glow_color()),
    ] {
        v(k, c.to_css());
    }
    for (k, c) in p.roles.list() {
        v(k, c.to_css());
    }
    for (i, c) in p.series.iter().enumerate() {
        v(&format!("series-{i}"), c.to_css());
    }
    v("font-display", ty.display.to_string());
    v("font-body", ty.body.to_string());
    v("font-mono", ty.mono.to_string());
    for (k, px) in [
        ("size-xs", ty.size_xs),
        ("size-sm", ty.size_sm),
        ("size-md", ty.size_md),
        ("size-lg", ty.size_lg),
        ("size-xl", ty.size_xl),
        ("size-xxl", ty.size_xxl),
        ("space-xs", t.spacing.xs),
        ("space-sm", t.spacing.sm),
        ("space-md", t.spacing.md),
        ("space-lg", t.spacing.lg),
        ("space-xl", t.spacing.xl),
        ("radius", t.shape.radius),
        ("chamfer", t.shape.chamfer),
        ("border-width", t.shape.border_width),
        ("control-height", t.shape.control_height),
        ("glow-blur", t.effects.glow_radius()),
    ] {
        v(k, format!("{px}px"));
    }
    v(
        "label-case",
        if ty.uppercase_labels {
            "uppercase"
        } else {
            "none"
        }
        .to_string(),
    );
    v("tracking", format!("{}em", ty.label_tracking));
    v(
        "motion-duration",
        format!("{}ms", t.motion.duration.as_millis()),
    );
    v("motion-delay", format!("{}ms", t.motion.delay.as_millis()));
    v("motion-easing", t.motion.easing.to_css());
    v(
        "scanlines",
        if t.effects.scanlines { "1" } else { "0" }.to_string(),
    );
    let chamfered = t.shape.kind() == ShapeKind::Chamfered;
    let glow_on = t.effects.glow_enabled();
    let (gc, gr) = (t.effects.glow_color().to_css(), t.effects.glow_radius());
    v(
        "glow-shadow",
        if glow_on && !chamfered {
            format!("0 0 {gr}px {gc}")
        } else {
            "0 0 0 0 transparent".into()
        },
    );
    v(
        "glow-filter",
        if glow_on && chamfered {
            format!("drop-shadow(0 0 {gr}px {gc})")
        } else {
            "none".into()
        },
    );
    v(
        "ring-inset",
        if chamfered {
            "0px".into()
        } else {
            format!("{}px", t.shape.border_width)
        },
    );
    v("clip", clip_path(&t.shape));
    v(
        "clip-sm",
        clip_path(&with_chamfer(&t.shape, t.shape.chamfer.min(6.0))),
    );
    v("clip-inner", clip_inner(&t.shape));
    v("clip-ring", clip_ring(&t.shape));
    v(
        "inner-radius",
        format!("{}px", (t.shape.radius - t.shape.border_width).max(0.0)),
    );
    v("hard-shadow", format!("{}px", t.effects.hard_shadow));
    v("scheme", if t.is_dark() { "dark" } else { "light" }.into());
    v("on-secondary-hover", t.on(p.secondary_hover).to_css());
    v("danger-hover", p.danger.lighten(0.15).to_css());
    v("danger-active", p.danger.darken(0.2).to_css());
    v("secondary-active", p.secondary.darken(0.2).to_css());
    for (k, c) in [
        ("primary", p.primary),
        ("secondary", p.secondary),
        ("info", p.info),
        ("success", p.success),
        ("warning", p.warning),
        ("danger", p.danger),
    ] {
        v(&format!("on-{k}-solid"), t.on(c).to_css());
        v(&format!("{k}-soft"), c.with_alpha(0.12).to_css());
    }
    s
}

/// Full stylesheet for a single theme (`:root` vars + components).
pub fn stylesheet(t: &Theme) -> String {
    format!(":root{{{}}}\n{}", theme_vars(t), COMPONENTS_CSS)
}

/// Stylesheet for runtime switching. The first theme is the `:root` default;
/// every theme (including the first) is addressable as
/// `:root[data-theme="<id>"]`.
pub fn multi_theme_stylesheet(themes: &[Theme]) -> String {
    let mut s = String::new();
    if let Some(first) = themes.first() {
        let _ = writeln!(s, ":root{{{}}}", theme_vars(first));
    }
    for t in themes {
        let _ = writeln!(s, ":root[data-theme=\"{}\"]{{{}}}", t.id(), theme_vars(t));
    }
    s.push_str(COMPONENTS_CSS);
    s
}

/// Inline script: exposes `dodaiTheme.next()/prev()/set(id)`, persists the
/// choice in `localStorage` and applies `data-theme` on `<html>` before paint.
/// Wire buttons with `data-dodai-theme-next` / `data-dodai-theme-prev`.
pub fn switcher_script(themes: &[Theme]) -> String {
    let ids = themes
        .iter()
        .map(|t| format!("\"{}\"", t.id()))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "(function(){{var ids=[{ids}],k='dodai-theme',d=document.documentElement;\
var i=Math.max(0,ids.indexOf(localStorage.getItem(k)));\
function set(n){{i=((n%ids.length)+ids.length)%ids.length;d.setAttribute('data-theme',ids[i]);\
try{{localStorage.setItem(k,ids[i])}}catch(e){{}}}}\
window.dodaiTheme={{next:function(){{set(i+1)}},prev:function(){{set(i-1)}},\
set:function(id){{var n=ids.indexOf(id);if(n>=0)set(n)}},current:function(){{return ids[i]}}}};\
set(i);\
document.addEventListener('click',function(e){{var t=e.target.closest&&e.target.closest('[data-dodai-theme-next],[data-dodai-theme-prev]');\
if(!t)return;t.hasAttribute('data-dodai-theme-next')?set(i+1):set(i-1)}});}})();"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai::themes;
    use dodai_core::themes::ThemeBuilder;

    #[test]
    fn vars_contain_brand_yellow() {
        let v = theme_vars(&dodai::themes::cyberpunk::yellow());
        assert!(v.contains("--dodai-primary:#fcee0a;"));
        assert!(v.contains("--dodai-secondary:#00f0ff;"));
        assert!(v.contains("--dodai-chamfer:16px;"));
        assert!(v.contains("--dodai-label-case:uppercase;"));
    }

    #[test]
    fn multi_has_every_theme() {
        let all = themes::all();
        let css = multi_theme_stylesheet(&all);
        for t in &all {
            assert!(css.contains(&format!("[data-theme=\"{}\"]", t.id())));
        }
        assert!(css.starts_with(":root{"));
    }

    #[test]
    fn css_is_safe_for_inline_style() {
        let css = multi_theme_stylesheet(&themes::all());
        assert!(!css.to_ascii_lowercase().contains("</style"));
    }

    #[test]
    fn braces_balanced() {
        let css = multi_theme_stylesheet(&themes::all());
        assert_eq!(css.matches('{').count(), css.matches('}').count());
    }

    #[test]
    fn components_only_use_defined_vars() {
        let defined = theme_vars(&dodai::themes::cyberpunk::yellow());
        let mut missing = vec![];
        for part in COMPONENTS_CSS.split("var(--dodai-").skip(1) {
            let name: String = part
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            if !defined.contains(&format!("--dodai-{name}:")) {
                missing.push(name);
            }
        }
        assert!(missing.is_empty(), "undefined vars: {missing:?}");
    }

    fn light(radius: f32) -> Theme {
        let mut t = sample_light();
        t.shape.radius = radius;
        t.meta.id = if radius > 0.0 {
            "test-light"
        } else {
            "test-square"
        };
        t
    }

    fn sample_light() -> Theme {
        use dodai_core::{Corners, Mode, Shape, Spacing, ThemeMeta, Typography};
        ThemeBuilder {
            meta: ThemeMeta {
                id: "test-light",
                name: "Test Light",
                family: "test",
                description: "",
                mode: Mode::Light,
            },
            mode: Mode::Light,
            background: 0xf6f7f9,
            surface: 0xffffff,
            raised: 0xeceef2,
            border: 0xd5d9e0,
            border_strong: 0xaab1bd,
            text: 0x14171c,
            muted: 0x4a5260,
            disabled: 0x8a92a0,
            primary: 0x1f5fd6,
            primary_hover: 0x1a50b5,
            on_primary: 0xffffff,
            secondary: 0x0a7d6e,
            on_secondary: 0xffffff,
            accent: 0xb45309,
            info: 0x1f6fd6,
            success: 0x15803d,
            warning: 0xb45309,
            danger: 0xc81e3a,
            series: [
                0x1f5fd6, 0x0a7d6e, 0xb45309, 0xc81e3a, 0x7c3aed, 0x0e7490, 0xbe185d, 0x4a5260,
            ],
            typography: Typography {
                display: "Inter",
                body: "Inter",
                mono: "Menlo",
                size_xs: 11.0,
                size_sm: 13.0,
                size_md: 15.0,
                size_lg: 18.0,
                size_xl: 24.0,
                size_xxl: 36.0,
                uppercase_labels: false,
                label_tracking: 0.0,
            },
            spacing: Spacing {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 20.0,
                xl: 32.0,
            },
            shape: Shape {
                corners: Corners::NONE,
                radius: 8.0,
                chamfer: 0.0,
                scoop: false,
                border_width: 1.0,
                control_height: 40.0,
            },
            glow: 0.0,
            glow_blur: 0.0,
            scanlines: false,
            hard_shadow: 2.0,
        }
        .build()
    }

    /// Evaluate a symbolic clip polygon at `w`x`h`.
    fn eval_clip(poly: &str, w: f32, h: f32) -> Vec<(f32, f32)> {
        fn tok(t: &str, full: f32) -> f32 {
            let t = t.trim();
            if t == "0" {
                0.0
            } else if t == "100%" {
                full
            } else if let Some(r) = t.strip_prefix("calc(100% - ") {
                full - r
                    .trim_end_matches(")")
                    .trim_end_matches("px")
                    .parse::<f32>()
                    .unwrap()
            } else {
                t.trim_end_matches("px").parse().unwrap()
            }
        }
        let inner = poly
            .strip_prefix("polygon(")
            .unwrap()
            .strip_suffix(')')
            .unwrap();
        let (mut pts, mut depth, mut cur) = (vec![], 0, String::new());
        for ch in inner.chars() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            if ch == ',' && depth == 0 {
                pts.push(std::mem::take(&mut cur));
            } else {
                cur.push(ch);
            }
        }
        pts.push(cur);
        pts.iter()
            .map(|p| {
                let p = p.trim();
                let split = if p.contains("calc(") && !p.starts_with("calc(") {
                    p.find(' ').unwrap()
                } else if p.starts_with("calc(") {
                    p.find(')').unwrap() + 1
                } else {
                    p.find(' ').unwrap()
                };
                (tok(&p[..split], w), tok(&p[split..], h))
            })
            .collect()
    }

    #[test]
    fn chamfered_clip_matches_outline() {
        for t in themes::all() {
            let c = theme_vars(&t);
            let want = t.shape.outline(240.0, 120.0);
            if t.shape.kind() == dodai_core::ShapeKind::Chamfered {
                let clip = clip_path(&t.shape);
                assert!(c.contains(&format!("--dodai-clip:{clip};")));
                assert_eq!(eval_clip(&clip, 240.0, 120.0), want, "{}", t.id());
            } else {
                assert!(c.contains("--dodai-clip:none;"));
            }
        }
    }

    #[test]
    fn cyberpunk_clip_is_bottom_right_only() {
        let t = dodai::themes::cyberpunk::yellow();
        assert_eq!(
            clip_path(&t.shape),
            "polygon(0 0,100% 0,100% calc(100% - 16px),calc(100% - 16px) 100%,0 100%)"
        );
    }

    #[test]
    fn rounded_and_square_have_no_clip() {
        for (t, r) in [(light(8.0), "8px"), (light(0.0), "0px")] {
            let v = theme_vars(&t);
            assert!(v.contains("--dodai-clip:none;") && v.contains("--dodai-clip-inner:none;"));
            assert!(v.contains(&format!("--dodai-radius:{r};")));
            assert!(v.contains("--dodai-glow-filter:none;"));
            assert!(v.contains("--dodai-scheme:light;"));
            assert!(v.contains("--dodai-clip-ring:polygon(0 0,0 0,0 0);"));
        }
    }

    #[test]
    fn components_defined_for_every_shape() {
        let mut themes = themes::all();
        themes.extend([light(8.0), light(0.0)]);
        for t in &themes {
            let defined = theme_vars(t);
            for part in COMPONENTS_CSS.split("var(--dodai-").skip(1) {
                let name: String = part
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .collect();
                assert!(
                    defined.contains(&format!("--dodai-{name}:")),
                    "{}: {name}",
                    t.id()
                );
            }
        }
        if let Ok(p) = std::env::var("DODAI_DUMP") {
            std::fs::write(p, multi_theme_stylesheet(&themes)).unwrap();
        }
    }

    #[test]
    fn glow_uses_filter_when_chamfered_shadow_otherwise() {
        let cp = theme_vars(&dodai::themes::cyberpunk::yellow());
        assert!(cp.contains("--dodai-glow-filter:drop-shadow("));
        assert!(cp.contains("--dodai-glow-shadow:0 0 0 0 transparent;"));
        let off = theme_vars(&dodai::themes::cyberpunk::yellow().with_glow_intensity(0.0));
        assert!(off.contains("--dodai-glow-filter:none;"));
    }

    #[test]
    fn script_lists_ids() {
        let js = switcher_script(&themes::all());
        assert!(js.contains("\"cyberpunk\",\"cyberpunk-cyan\",\"cyberpunk-red\""));
    }
}
