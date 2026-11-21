//! # dodai-topcoat
//!
//! Generates the Dodai UI **stylesheet** for server-rendered web UIs
//! (built for [Topcoat](https://crates.io/crates/topcoat), works with any
//! framework): CSS custom properties for every token, a themed component layer
//! (`.dodai-btn`, `.dodai-panel`, `.dodai-input`, `.dodai-tag`, `.dodai-tabs`, `.dodai-table`, …
//! with chamfered cut-corners via `clip-path`) and a tiny client-side switcher.
//!
//! ```
//! use dodai_core::themes;
//! let css = dodai_topcoat::multi_theme_stylesheet(&themes::all());
//! assert!(css.contains("[data-theme=\"cyberpunk-cyan\"]"));
//! ```
//!
//! In a Topcoat page, inline it (it never contains `</style`):
//! `<style>(css)</style>` and call `<script>(dodai_topcoat::switcher_script(..))</script>`.

pub mod html;

use dodai_core::{Kit, Shape, ShapeKind, Theme};
use std::fmt::Write;

/// Component layer; only references `var(--dodai-*)`.
pub const COMPONENTS_CSS: &str = include_str!("components.css");

/// Google Fonts stylesheet URL for the fallback display face (Rajdhani) and mono.
pub const GOOGLE_FONTS_URL: &str =
    "https://fonts.googleapis.com/css2?family=Rajdhani:wght@400;500;600;700&family=JetBrains+Mono:wght@400;600&display=swap";

const PROBE: f32 = 4096.0;

/// `v` px, rounded to hundredths so scooped arcs stay short in the stylesheet.
/// The `--dodai-panel-*` variables (without the prefix) that frame a panel like `frame`: set them
/// on an element to give the panels inside their own border.
///
/// The hard shadow is the panel's `.dodai-panel__shade` child: the panel's own shape, offset
/// and filled (a CSS shadow would follow the content or ignore cut corners).
pub fn panel_frame_vars(t: &Theme, frame: &dodai_core::PanelFrame) -> [(&'static str, String); 6] {
    let chamfered = t.shape.kind() == ShapeKind::Chamfered;
    let (bd, bw) = frame
        .outline
        .map_or(("transparent".to_string(), 0.0), |(c, w)| (c.to_css(), w));
    let (sc, (sx, sy)) = frame
        .shadow
        .map_or(("transparent".to_string(), (0.0, 0.0)), |(c, o)| {
            (c.to_css(), o)
        });
    [
        ("panel-bd", bd),
        ("panel-bw", format!("{bw}px")),
        (
            "panel-ring",
            format!("{}px", if chamfered { 0.0 } else { bw }),
        ),
        ("panel-sc", sc),
        ("panel-sx", format!("{sx}px")),
        ("panel-sy", format!("{sy}px")),
    ]
}

fn px(v: f32) -> String {
    format!("{}px", (v * 100.0).round() / 100.0)
}

/// Symbolic CSS coordinate for a value measured on a `PROBE`-sized box: the box edges, else a
/// distance from the nearer one (so straight cuts and scooped arcs both scale with the box).
fn coord(v: f32) -> String {
    let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
    if near(v, 0.0) {
        "0".into()
    } else if near(v, PROBE) {
        "100%".into()
    } else if v < PROBE / 2.0 {
        px(v)
    } else {
        format!("calc(100% - {})", px(PROBE - v))
    }
}

/// Points of `shape`'s outline as CSS `x y` pairs (percent + px).
fn clip_points(shape: &Shape) -> Vec<String> {
    shape
        .outline(PROBE, PROBE)
        .into_iter()
        .map(|(x, y)| format!("{} {}", coord(x), coord(y)))
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
    let pts = inner.outline(PROBE, PROBE).into_iter().map(|(x, y)| {
        // The inner outline sits `bw` inside the box on every side.
        let f = |v: f32| {
            let near = |a: f32, b: f32| (a - b).abs() < 1e-3;
            if near(v, 0.0) {
                px(bw)
            } else if near(v, PROBE) {
                format!("calc(100% - {})", px(bw))
            } else if v < PROBE / 2.0 {
                px(v + bw)
            } else {
                format!("calc(100% - {})", px(PROBE - v + bw))
            }
        };
        format!("{} {}", f(x), f(y))
    });
    let mut all = clip_points(shape);
    all.extend(pts);
    format!("polygon(evenodd,{})", all.join(","))
}

/// Custom properties (an inline `style`) that crop a `.dodai-btn` the way `t`'s design crops
/// buttons: cut corners and ticket stubs through the clip paths, tight corners through the
/// radii. Empty for a crop that only draws a tick or a slot.
pub fn crop_style(t: &Theme, crop: dodai_core::ButtonCrop) -> String {
    use dodai_core::ButtonCrop as B;
    let sh = &t.shape;
    let cut = |shape: Shape| {
        format!(
            "--dodai-clip:{};--dodai-clip-ring:{};--dodai-clip-inner:{};--dodai-ring-inset:0px;--dodai-radius:0;--dodai-inner-radius:0;",
            clip_path(&shape),
            clip_ring(&shape),
            clip_inner(&shape)
        )
    };
    match crop {
        B::CutTicked { corners, .. } => cut(Shape {
            corners,
            // Steep 45° cut: the largest a control of the design's height allows (a third of it,
            // as `Shape::outline` clamps).
            chamfer: sh.chamfer.max(sh.control_height / 3.0),
            scoop: false,
            ..sh.clone()
        }),
        B::Scooped { corners, size } => cut(Shape {
            corners,
            chamfer: size,
            scoop: true,
            ..sh.clone()
        }),
        B::Squared {
            corners,
            radius,
            rest,
        } => {
            let pick = |on: bool, inset: f32| px((if on { radius } else { rest } - inset).max(0.0));
            let radii = |inset: f32| {
                format!(
                    "{} {} {} {}",
                    pick(corners.top_left, inset),
                    pick(corners.top_right, inset),
                    pick(corners.bottom_right, inset),
                    pick(corners.bottom_left, inset)
                )
            };
            format!(
                "--dodai-radius:{};--dodai-inner-radius:{};",
                radii(0.0),
                radii(sh.border_width)
            )
        }
        B::Slot { .. } => String::new(),
    }
}

/// `svg` as a CSS `url(...)` value (a percent-encoded `data:` URI), for `background-image`.
pub fn svg_url(svg: &str) -> String {
    let mut out = String::with_capacity(svg.len() + 32);
    out.push_str("url(\"data:image/svg+xml,");
    for b in svg.bytes() {
        match b {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'~'
            | b'/'
            | b':'
            | b'='
            | b','
            | b'\''
            | b'('
            | b')'
            | b'!'
            | b'*' => out.push(b as char),
            _ => {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out.push_str("\")");
    out
}

/// The surface `fx` for a `class` surface of `size` px as a CSS `background-image` value (a
/// data-URI SVG drawn by the same `dodai-fx` geometry as every native toolkit): textures,
/// lit lines, wash, brackets and the rest, frozen at one moment. `None` when the theme draws
/// nothing of it. Put it on an element of exactly that size, over the element's own fill.
pub fn surface_background(
    theme: &Theme,
    fx: dodai_core::ResolvedSurface,
    class: dodai_core::SurfaceClass,
    size: (f32, f32),
) -> Option<String> {
    let seed = (size.0 as i32).wrapping_mul(31) ^ size.1 as i32;
    dodai_core::surface_svg(
        theme,
        class,
        &dodai_core::Surface::INHERIT,
        fx,
        size,
        (seed, 0.0),
    )
    .map(|(svg, _)| svg_url(&svg))
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
        ("highlight", t.effects.highlight),
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
        ("tick-gap", t.tick_gap()),
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
    for (k, px) in dodai_core::size_entries(&t.sizes()) {
        v(&format!("sz-{k}"), format!("{px}px"));
    }
    let nav = t.navigation();
    for (k, px) in dodai_core::nav_entries(&nav) {
        v(&format!("nav-{k}"), format!("{px}px"));
    }
    v("nav-pill", if nav.pill { "1" } else { "0" }.to_string());
    let look = t.look();
    for (k, on) in [
        ("tonal", look.tonal),
        ("soft-elevation", look.soft_elevation),
        ("floating-label", look.floating_label),
        ("sink", look.sink),
        ("luminous", look.luminous),
        ("cut-corners", look.cut_corners),
        ("accent-border", look.accent_border),
        ("segmented-tabs", look.segmented_tabs),
    ] {
        v(&format!("look-{k}"), if on { "1" } else { "0" }.to_string());
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
    // The tune's shadow direction.
    let (hx, hy) = t.hard_offset().unwrap_or((0.0, 0.0));
    v("hard-x", format!("{hx}px"));
    v("hard-y", format!("{hy}px"));
    // Panels: the frame's contour and the hard shadow on its accent corner.
    for (k, val) in panel_frame_vars(t, &t.panel_frame(false)) {
        v(k, val);
    }
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
    s.push_str(&kit_vars(t));
    s
}

/// Whether a [`Kit`] reacts to the pointer (and so has hover variables).
fn kit_hovers(kit: Kit) -> bool {
    matches!(
        kit,
        Kit::Chip { .. } | Kit::Row { .. } | Kit::Page { .. } | Kit::Segment { .. } | Kit::Select
    )
}

/// `--dodai-kit-<name>-*` declarations for every [`Kit`] of `t`: `bg`, `bd`, `fg`, `px`, `py`,
/// `glow`, and `hover-bg`, `hover-bd`, `hover-fg` for the clickable ones. A component picks one
/// with [`kit_style`].
pub fn kit_vars(t: &Theme) -> String {
    let mut s = String::new();
    let paint = |c: Option<dodai_core::Color>| c.map_or("transparent".to_string(), |c| c.to_css());
    let glow = |on: bool| {
        if on && t.effects.glow_enabled() {
            format!(
                "0 0 {}px {}",
                t.effects.glow_radius(),
                t.effects.glow_color().to_css()
            )
        } else {
            "0 0 0 0 transparent".to_string()
        }
    };
    let lift = if let Some((hx, hy)) = t.hard_offset() {
        format!("{hx}px {hy}px 0 {}", t.hard_shadow_color().to_css())
    } else {
        "0 1px 3px rgba(0,0,0,.3),0 4px 8px 3px rgba(0,0,0,.15)".to_string()
    };
    for kit in Kit::all() {
        let n = kit.name();
        let st = t.kit(kit, false);
        let shadow = if st.lift {
            lift.as_str()
        } else {
            "0 0 0 0 transparent"
        };
        let _ = write!(s, "--dodai-kit-{n}-lift:{shadow};");
        let _ = write!(
            s,
            "--dodai-kit-{n}-bg:{};--dodai-kit-{n}-bd:{};--dodai-kit-{n}-fg:{};\
--dodai-kit-{n}-px:{}px;--dodai-kit-{n}-py:{}px;--dodai-kit-{n}-glow:{};--dodai-kit-{n}-r:{};",
            paint(st.fill),
            paint(t.outline(st.border, st.fill.is_some()).map(|o| o.0)),
            st.fg.to_css(),
            st.padding.0,
            st.padding.1,
            glow(st.glow),
            if st.pill && t.shape.kind() == ShapeKind::Rounded {
                "9999px"
            } else {
                "var(--dodai-radius)"
            },
        );
        if kit_hovers(kit) {
            let h = t.kit(kit, true);
            let _ = write!(
                s,
                "--dodai-kit-{n}-hover-bg:{};--dodai-kit-{n}-hover-bd:{};--dodai-kit-{n}-hover-fg:{};",
                paint(h.fill),
                paint(h.border),
                h.fg.to_css(),
            );
        }
    }
    s
}

/// The inline `style` value that makes an element of `.dodai-kit` look like `kit`: it points
/// the generic variables of the stylesheet at the variables [`kit_vars`] declared for it.
pub fn kit_style(kit: Kit) -> String {
    let n = kit.name();
    let mut s = format!(
        "--kb:var(--dodai-kit-{n}-bg);--kd:var(--dodai-kit-{n}-bd);--kf:var(--dodai-kit-{n}-fg);\
--kx:var(--dodai-kit-{n}-px);--ky:var(--dodai-kit-{n}-py);--kg:var(--dodai-kit-{n}-glow);--kr:var(--dodai-kit-{n}-r);\
--kl:var(--dodai-kit-{n}-lift);"
    );
    if kit_hovers(kit) {
        let _ = write!(
            s,
            "--kbh:var(--dodai-kit-{n}-hover-bg);--kdh:var(--dodai-kit-{n}-hover-bd);--kfh:var(--dodai-kit-{n}-hover-fg);"
        );
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

/// Inline script: roving focus for every `.dodai-tabs` strip. The arrows, Home and End move to
/// the next tab that is not `.dodai-tab--disabled` (the rule of `dodai_core::roving`, mirrored
/// for right-to-left), focus it and click it so the tab's own attributes select it. Load once
/// per page; it listens on the document.
pub const TABS_SCRIPT: &str = "(function(){document.addEventListener('keydown',function(e){\
var s=e.target.closest&&e.target.closest('.dodai-tabs');if(!s||e.target.closest('.dodai-tab__close'))return;\
var t=[].slice.call(s.querySelectorAll('.dodai-tab:not(.dodai-tab--add)')),\
on=function(i){return !t[i].classList.contains('dodai-tab--disabled')},\
at=t.indexOf(e.target.closest('.dodai-tab')),rtl=getComputedStyle(s).direction==='rtl',k=e.key;\
if(at<0)return;if(rtl&&(k==='ArrowLeft'||k==='ArrowRight'))k=k==='ArrowLeft'?'ArrowRight':'ArrowLeft';\
var to=-1,i;if(k==='ArrowRight'){for(i=at+1;i<t.length;i++)if(on(i)){to=i;break}}\
else if(k==='ArrowLeft'){for(i=at-1;i>=0;i--)if(on(i)){to=i;break}}\
else if(k==='Home'){for(i=0;i<t.length;i++)if(on(i)){to=i;break}}\
else if(k==='End'){for(i=t.length-1;i>=0;i--)if(on(i)){to=i;break}}\
if(to<0||to===at)return;e.preventDefault();t[to].focus();t[to].click()})})();";

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

/// Inline style that makes one surface see-through: frosted glass from the app-wide mix
/// `active` (with the component's own `over` on top) and the component's fill opacity, as the
/// `--g*` variables the `.dodai-glass` rules read. Put it with the class `dodai-glass` on a
/// `.dodai-btn`, `.dodai-panel` or `.dodai-kit` element. Empty when the fill stays as designed.
pub fn glass_style(
    theme: &Theme,
    class: dodai_core::SurfaceClass,
    over: &dodai_core::Surface,
    active: dodai_core::ResolvedSurface,
) -> String {
    glass_vars(&dodai_core::surface_glass(theme, class, over, active))
}

/// The `--g*` variables of one [`Glass`](dodai_core::Glass) (see [`glass_style`]).
pub fn glass_vars(g: &dodai_core::Glass) -> String {
    if g.is_solid() {
        return String::new();
    }
    let mut s = format!(
        "--ga:{};--gc:{};--gk:{};--go:{};",
        g.amount, g.clear, g.keep, g.opacity
    );
    if g.frosted() {
        s.push_str(&format!(
            "--gf:blur({}px) saturate({});",
            g.blur * g.amount,
            1.0 + (g.saturation - 1.0) * g.amount
        ));
    }
    s
}

/// The theme's canvas mesh (see [`dodai_core::mesh_pixels`]) as a CSS `background-image` value of
/// radial gradients, stretched over the element like the picture the native toolkits stretch
/// over the window. `None` for a design with a flat canvas.
pub fn backdrop_css(t: &Theme) -> Option<String> {
    let style = &t.surface_style;
    if style.backdrop <= 0.0 {
        return None;
    }
    let (w, h) = dodai_core::MESH_SIZE;
    let aspect = h as f32 / w as f32;
    // Stops sampled along the radius; CSS draws the first image on top, so reverse the blobs.
    let layers: Vec<String> = dodai_core::mesh_blobs(style.wash, style.backdrop)
        .iter()
        .rev()
        .map(|b| {
            let stops: Vec<String> = (0..=8)
                .map(|i| {
                    let d = i as f32 / 8.0;
                    format!(
                        "{} {:.1}%",
                        b.color.with_alpha(b.alpha_at(d)).to_css(),
                        d * 100.0
                    )
                })
                .collect();
            format!(
                "radial-gradient({:.2}% {:.2}% at {:.1}% {:.1}%,{})",
                b.r * aspect * 100.0,
                b.r * 100.0,
                b.x * 100.0,
                b.y * 100.0,
                stops.join(",")
            )
        })
        .collect();
    Some(layers.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::themes;
    use dodai_core::themes::ThemeBuilder;

    /// Each design's crop turns into the custom properties its kind needs.
    #[test]
    fn crop_styles_follow_the_crop_kind() {
        use dodai_core::{ButtonCrop as B, Crop};
        let mut seen = 0;
        for t in themes::all() {
            for end in Crop::ALL {
                let Some(crop) = t.button_crop(end) else {
                    continue;
                };
                let style = crop_style(&t, crop);
                match crop {
                    B::CutTicked { .. } | B::Scooped { .. } => {
                        assert!(
                            style.contains("--dodai-clip-ring:polygon(evenodd"),
                            "{style}"
                        );
                        assert!(style.contains("--dodai-radius:0"), "{style}");
                    }
                    B::Squared { .. } => {
                        assert!(style.starts_with("--dodai-radius:"), "{style}");
                        assert!(!style.contains("polygon"), "{style}");
                    }
                    B::Slot { .. } => assert!(style.is_empty()),
                }
                seen += 1;
            }
        }
        assert!(seen > 0);
    }

    /// Every `var(--dodai-kit-…)` a kit's inline style points at is declared for the theme.
    #[test]
    fn kit_styles_only_reference_declared_variables() {
        for t in themes::all() {
            let vars = theme_vars(&t);
            for kit in Kit::all() {
                let style = kit_style(kit);
                for part in style.split("var(--").skip(1) {
                    let name = part.split(')').next().unwrap();
                    assert!(
                        vars.contains(&format!("--{name}:")),
                        "{} {}: --{name} is not declared",
                        t.id(),
                        kit.name()
                    );
                }
            }
        }
    }

    #[test]
    fn only_clickable_kits_get_hover_variables() {
        let t = themes::all().remove(0);
        let vars = theme_vars(&t);
        assert!(vars.contains("--dodai-kit-chip-on-hover-bg:"));
        assert!(vars.contains("--dodai-kit-select-hover-bd:"));
        assert!(!vars.contains("--dodai-kit-card-filled-hover-bg:"));
        assert!(kit_style(Kit::Chip { selected: false }).contains("--kbh:"));
        assert!(!kit_style(Kit::Code).contains("--kbh:"));
    }

    #[test]
    fn pills_round_fully_only_on_rounded_designs() {
        let pill = |t: &Theme| {
            let key = "--dodai-kit-avatar-r:";
            let v = theme_vars(t);
            let at = v.find(key).unwrap() + key.len();
            v[at..].split(';').next().unwrap().to_string()
        };
        for t in themes::all() {
            let want = if t.shape.kind() == ShapeKind::Rounded {
                "9999px"
            } else {
                "var(--dodai-radius)"
            };
            assert_eq!(pill(&t), want, "{}", t.id());
        }
    }

    #[test]
    fn glow_only_when_the_theme_glows() {
        for t in themes::all() {
            let v = theme_vars(&t);
            let key = "--dodai-kit-step-current-glow:";
            let at = v.find(key).unwrap() + key.len();
            let glow = v[at..].split(';').next().unwrap();
            if !t.effects.glow_enabled() {
                assert_eq!(glow, "0 0 0 0 transparent", "{}", t.id());
            }
        }
    }

    #[test]
    fn vars_contain_brand_yellow() {
        let v = theme_vars(&dodai_core::themes::cyberpunk::yellow());
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
        let defined = theme_vars(&dodai_core::themes::cyberpunk::yellow());
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
        use dodai_core::{Mode, ThemeMeta};
        ThemeBuilder::neutral(ThemeMeta {
            id: "test-light",
            name: "Test Light",
            family: "test",
            description: "",
            mode: Mode::Light,
        })
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
    fn a_tuned_theme_flows_into_the_stylesheet() {
        let t = dodai_core::themes::material::light();
        let square = t.tuned(&dodai_core::ShapeTune {
            roundness: 0.0,
            ..dodai_core::ShapeTune::DEFAULT
        });
        assert!(!theme_vars(&t).contains("--dodai-radius:0px;"));
        assert!(theme_vars(&square).contains("--dodai-radius:0px;"));
    }

    #[test]
    fn surface_background_is_a_css_url() {
        let t = dodai_core::themes::cyberpunk::yellow();
        let fx = dodai_core::ResolvedSurface {
            texture: t
                .surface_textures()
                .iter()
                .copied()
                .find(|x| *x != dodai_core::Texture::None)
                .unwrap(),
            strength: 0.8,
            targets: dodai_core::SurfaceTargets::ALL,
            ..dodai_core::ResolvedSurface::NONE
        };
        let css = surface_background(&t, fx, dodai_core::SurfaceClass::Button, (160.0, 80.0))
            .expect("the design's own texture");
        assert!(css.starts_with("url(\"data:image/svg+xml,%3Csvg"));
        assert!(css.ends_with("%3C/svg%3E\")") && !css.contains(' ') && !css.contains('#'));
        assert!(surface_background(
            &t,
            dodai_core::ResolvedSurface::NONE,
            dodai_core::SurfaceClass::Button,
            (160.0, 80.0)
        )
        .is_none());
    }

    #[test]
    fn scooped_clip_follows_the_arc_on_any_box() {
        let mut t = dodai_core::themes::cyberpunk::yellow();
        t.shape.scoop = true;
        let clip = clip_path(&t.shape);
        let pts = |w: f32, h: f32| eval_clip(&clip, w, h);
        for (w, h) in [(240.0, 120.0), (90.0, 60.0)] {
            let (got, want) = (pts(w, h), t.shape.outline(w, h));
            assert_eq!(got.len(), want.len());
            for (g, v) in got.iter().zip(&want) {
                assert!(
                    (g.0 - v.0).abs() < 0.05 && (g.1 - v.1).abs() < 0.05,
                    "{g:?} {v:?}"
                );
            }
        }
        assert!(
            clip.matches(',').count() > 8,
            "arc points are in the polygon"
        );
        let ring = clip_ring(&t.shape);
        assert!(ring.starts_with("polygon(evenodd,") && ring.matches(',').count() > 16);
    }

    #[test]
    fn cyberpunk_clip_is_bottom_right_only() {
        let t = dodai_core::themes::cyberpunk::yellow();
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
        let cp = theme_vars(&dodai_core::themes::cyberpunk::yellow());
        assert!(cp.contains("--dodai-glow-filter:drop-shadow("));
        assert!(cp.contains("--dodai-glow-shadow:0 0 0 0 transparent;"));
        let off = theme_vars(&dodai_core::themes::cyberpunk::yellow().with_glow_intensity(0.0));
        assert!(off.contains("--dodai-glow-filter:none;"));
    }

    #[test]
    fn script_lists_ids() {
        let js = switcher_script(&themes::all());
        assert!(js.contains("\"cyberpunk\",\"cyberpunk-cyan\",\"cyberpunk-red\""));
    }

    #[test]
    fn glass_is_empty_for_a_solid_fill_and_frost_adds_the_backdrop_filter() {
        use dodai_core::{ResolvedSurface, Surface, SurfaceClass};
        for t in themes::all() {
            let solid = glass_style(
                &t,
                SurfaceClass::Card,
                &Surface::INHERIT,
                ResolvedSurface::NONE,
            );
            assert!(solid.is_empty(), "{}", t.id());
            let clear = glass_style(
                &t,
                SurfaceClass::Button,
                &Surface::new().opacity(0.5),
                ResolvedSurface::NONE,
            );
            assert!(
                clear.contains("--go:0.5") && !clear.contains("--gf"),
                "{}",
                t.id()
            );
            let frost = glass_style(
                &t,
                SurfaceClass::Panel,
                &Surface::new().frost(1.0),
                ResolvedSurface::NONE,
            );
            assert!(
                frost.contains("--ga:1;") && frost.contains("--gf:blur("),
                "{}",
                t.id()
            );
        }
        assert!(COMPONENTS_CSS.contains(".dodai-glass{"));
    }

    #[test]
    fn only_a_meshed_canvas_has_a_backdrop() {
        for t in themes::all() {
            let css = backdrop_css(&t);
            assert_eq!(css.is_some(), t.surface_style.backdrop > 0.0, "{}", t.id());
            if let Some(css) = css {
                assert_eq!(css.matches("radial-gradient(").count(), 4, "{}", t.id());
            }
        }
    }
}
