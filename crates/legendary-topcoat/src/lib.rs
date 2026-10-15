//! # legendary-topcoat
//!
//! Generates the Legendary Themes **stylesheet** for server-rendered web UIs
//! (built for [Topcoat](https://crates.io/crates/topcoat), works with any
//! framework): CSS custom properties for every token, a themed component layer
//! (`.lg-btn`, `.lg-panel`, `.lg-input`, `.lg-tag`, `.lg-tabs`, `.lg-table`, …
//! with chamfered cut-corners via `clip-path`) and a tiny client-side switcher.
//!
//! ```
//! use legendary_core::themes;
//! let css = legendary_topcoat::multi_theme_stylesheet(&themes::all());
//! assert!(css.contains("[data-theme=\"cyberpunk-netrunner\"]"));
//! ```
//!
//! In a Topcoat page, inline it (it never contains `</style`):
//! `<style>(css)</style>` and call `<script>(legendary_topcoat::switcher_script(..))</script>`.

use legendary_core::Theme;
use std::fmt::Write;

/// Component layer; only references `var(--lg-*)`.
pub const COMPONENTS_CSS: &str = include_str!("components.css");

/// Google Fonts stylesheet URL for the fallback display face (Rajdhani) and mono.
pub const GOOGLE_FONTS_URL: &str =
    "https://fonts.googleapis.com/css2?family=Rajdhani:wght@400;500;600;700&family=JetBrains+Mono:wght@400;600&display=swap";

/// `--lg-*` custom property declarations (no selector) for one theme.
pub fn theme_vars(t: &Theme) -> String {
    let p = &t.palette;
    let ty = &t.typography;
    let mut s = String::new();
    let mut v = |k: &str, val: String| {
        let _ = write!(s, "--lg-{k}:{val};");
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
        "scanlines",
        if t.effects.scanlines { "1" } else { "0" }.to_string(),
    );
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

/// Inline script: exposes `legendaryTheme.next()/prev()/set(id)`, persists the
/// choice in `localStorage` and applies `data-theme` on `<html>` before paint.
/// Wire buttons with `data-lg-theme-next` / `data-lg-theme-prev`.
pub fn switcher_script(themes: &[Theme]) -> String {
    let ids = themes
        .iter()
        .map(|t| format!("\"{}\"", t.id()))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "(function(){{var ids=[{ids}],k='legendary-theme',d=document.documentElement;\
var i=Math.max(0,ids.indexOf(localStorage.getItem(k)));\
function set(n){{i=((n%ids.length)+ids.length)%ids.length;d.setAttribute('data-theme',ids[i]);\
try{{localStorage.setItem(k,ids[i])}}catch(e){{}}}}\
window.legendaryTheme={{next:function(){{set(i+1)}},prev:function(){{set(i-1)}},\
set:function(id){{var n=ids.indexOf(id);if(n>=0)set(n)}},current:function(){{return ids[i]}}}};\
set(i);\
document.addEventListener('click',function(e){{var t=e.target.closest&&e.target.closest('[data-lg-theme-next],[data-lg-theme-prev]');\
if(!t)return;t.hasAttribute('data-lg-theme-next')?set(i+1):set(i-1)}});}})();"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use legendary_core::themes;

    #[test]
    fn vars_contain_brand_yellow() {
        let v = theme_vars(&Theme::default());
        assert!(v.contains("--lg-primary:#fcee0a;"));
        assert!(v.contains("--lg-secondary:#00f0ff;"));
        assert!(v.contains("--lg-chamfer:16px;"));
        assert!(v.contains("--lg-label-case:uppercase;"));
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
        let defined = theme_vars(&Theme::default());
        let mut missing = vec![];
        for part in COMPONENTS_CSS.split("var(--lg-").skip(1) {
            let name: String = part
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect();
            if !defined.contains(&format!("--lg-{name}:")) {
                missing.push(name);
            }
        }
        assert!(missing.is_empty(), "undefined vars: {missing:?}");
    }

    #[test]
    fn script_lists_ids() {
        let js = switcher_script(&themes::all());
        assert!(js.contains("\"cyberpunk\",\"cyberpunk-netrunner\",\"cyberpunk-samurai\""));
    }
}
