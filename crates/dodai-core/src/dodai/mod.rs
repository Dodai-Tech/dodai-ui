//! **Dodai**: frosted glass and gradients. Deep indigo ink on a pearl canvas (or lavender-white
//! on deep night), 16 px controls, 30 px containers, hairline borders, tinted soft shadows and a
//! violet-through-blue-to-teal gradient as the signature accent. Token values are Dodai's own
//! (see [`tokens`](crate::dodai::tokens)).

use crate::themes::ThemeBuilder;
use crate::{Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};

mod spec;
pub mod surface;
pub mod tokens;
pub use spec::{shape_presets, shape_system, shape_system_compact, Dodai, SHAPE_SCALE, SPEC};

/// The `sm` duration on the soft in-out easing.
fn motion() -> crate::Motion {
    let (a, b, c, d) = tokens::EASE_IN_OUT;
    crate::Motion::new(
        std::time::Duration::from_millis(tokens::DURATION_SM),
        crate::Easing::CubicBezier(a, b, c, d),
    )
    .personality(crate::motion::Personality::Smooth)
}

fn typography() -> Typography {
    // Type scale steps at the design width: 2xs, xs, sm, md, lg, xl.
    let at = |step| {
        tokens::type_size(step, tokens::DESIGN_WIDTH)
            .unwrap_or(16.0)
            .round()
    };
    Typography {
        display: tokens::FONT_STACK,
        body: tokens::FONT_STACK,
        mono: "SF Mono, Menlo, Consolas, monospace",
        size_xs: at("2xs"),
        size_sm: at("xs"),
        size_md: at("sm"),
        size_lg: at("md"),
        size_xl: at("lg"),
        size_xxl: at("xl"),
        uppercase_labels: false,
        label_tracking: 0.0,
    }
}

/// Static spacing: `xs … xl` = 6 / 10 / 16 / 24 / 40.
fn spacing() -> Spacing {
    let s = |n| tokens::spacing_static(n).unwrap_or(0.0);
    Spacing {
        xs: s("xs"),
        sm: s("sm"),
        md: s("md"),
        lg: s("lg"),
        xl: s("xl"),
    }
}

fn shape() -> Shape {
    Shape {
        corners: Corners::NONE,
        // The `xl` radius is the primary look of interactive controls.
        radius: tokens::radius("xl").unwrap_or(12.0),
        chamfer: 0.0,
        scoop: false,
        border_width: tokens::BORDER_WIDTH,
        control_height: 48.0,
    }
}

/// The solid colours of one mode: each token composited over its own canvas, because a theme
/// palette holds opaque colours. The alpha originals are in [`tokens`].
struct Ink {
    canvas: u32,
    surface: u32,
    primary: u32,
    primary_hover: u32,
    on_primary: u32,
    higher: u32,
    high: u32,
    medium: u32,
    low: u32,
    lower: u32,
    info: u32,
    success: u32,
    warning: u32,
    error: u32,
}

fn rgb(c: crate::Color) -> u32 {
    (u32::from(c.r) << 16) | (u32::from(c.g) << 8) | u32::from(c.b)
}

fn ink(t: &tokens::ColorTokens) -> Ink {
    let canvas = t.canvas.color();
    let over = |h: tokens::Hsla| rgb(h.over(canvas));
    Ink {
        canvas: rgb(canvas),
        surface: rgb(t.surface.color()),
        primary: rgb(t.ink.color()),
        // Hover: the primary eased 22 % towards the canvas.
        primary_hover: rgb(t.ink.color().mix(canvas, 0.22)),
        on_primary: rgb(canvas),
        higher: over(t.ink_90),
        high: over(t.ink_75),
        medium: over(t.ink_60),
        low: over(t.ink_45),
        lower: over(t.ink_25),
        info: rgb(t.info.solid.color()),
        success: rgb(t.success.solid.color()),
        warning: rgb(t.warning.solid.color()),
        error: rgb(t.danger.solid.color()),
    }
}

/// Rotates the hue of `c` by `deg` degrees, keeping saturation and lightness (so contrast holds).
pub(crate) fn rotate_hue(c: crate::Color, deg: f32) -> crate::Color {
    if deg == 0.0 {
        return c;
    }
    let (r, g, b) = (
        f32::from(c.r) / 255.0,
        f32::from(c.g) / 255.0,
        f32::from(c.b) / 255.0,
    );
    let (mx, mn) = (r.max(g).max(b), r.min(g).min(b));
    let l = (mx + mn) / 2.0;
    let d = mx - mn;
    if d < 1e-6 {
        return c;
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if mx == r {
        ((g - b) / d).rem_euclid(6.0)
    } else if mx == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } * 60.0;
    let h = (h + deg).rem_euclid(360.0);
    let at = |l: f32| hsl_to_color(h, s, l.clamp(0.0, 1.0), c.a);
    // Equal lightness is not equal brightness across hues (green is brighter than blue), so
    // search the lightness that keeps the original luminance and with it the text contrast.
    let target = c.luminance();
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    for _ in 0..14 {
        let mid = (lo + hi) / 2.0;
        if at(mid).luminance() < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    at((lo + hi) / 2.0)
}

fn hsl_to_color(h: f32, s: f32, l: f32, a: u8) -> crate::Color {
    let k = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = k * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - k / 2.0;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (k, x, 0.0),
        1 => (x, k, 0.0),
        2 => (0.0, k, x),
        3 => (0.0, x, k),
        4 => (x, 0.0, k),
        _ => (k, 0.0, x),
    };
    let to = |v: f32| ((v + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    crate::Color {
        r: to(r),
        g: to(g),
        b: to(b),
        a,
    }
}

/// An accent variant: the whole ink, canvas and gradient turned to another hue. Status colours
/// (info, success, warning, error) keep their meaning.
struct Accent {
    id: &'static str,
    dark_id: &'static str,
    name: &'static str,
    dark_name: &'static str,
    /// Degrees added to every hue of the signature violet.
    hue: f32,
}

const ACCENTS: [Accent; 4] = [
    Accent {
        id: "dodai-ocean",
        dark_id: "dodai-ocean-dark",
        name: "Dodai Ocean · Light",
        dark_name: "Dodai Ocean · Dark",
        hue: -48.0,
    },
    Accent {
        id: "dodai-forest",
        dark_id: "dodai-forest-dark",
        name: "Dodai Forest · Light",
        dark_name: "Dodai Forest · Dark",
        hue: -105.0,
    },
    Accent {
        id: "dodai-rose",
        dark_id: "dodai-rose-dark",
        name: "Dodai Rose · Light",
        dark_name: "Dodai Rose · Dark",
        hue: 78.0,
    },
    Accent {
        id: "dodai-sunset",
        dark_id: "dodai-sunset-dark",
        name: "Dodai Sunset · Light",
        dark_name: "Dodai Sunset · Dark",
        hue: 145.0,
    },
];

fn build(mode: Mode) -> Theme {
    build_hue(mode, None)
}

fn build_hue(mode: Mode, accent: Option<&Accent>) -> Theme {
    let dark = mode == Mode::Dark;
    let hue = accent.map_or(0.0, |a| a.hue);
    let c = &ink(if dark { &tokens::DARK } else { &tokens::LIGHT });
    let h = |v: u32| {
        let c = rotate_hue(
            crate::Color {
                r: (v >> 16) as u8,
                g: (v >> 8) as u8,
                b: v as u8,
                a: 255,
            },
            hue,
        );
        rgb(c)
    };
    let mut t = ThemeBuilder {
        meta: ThemeMeta {
            id: match (accent, dark) {
                (Some(a), true) => a.dark_id,
                (Some(a), false) => a.id,
                (None, true) => "dodai-dark",
                (None, false) => "dodai",
            },
            name: match (accent, dark) {
                (Some(a), true) => a.dark_name,
                (Some(a), false) => a.name,
                (None, true) => "Dodai · Dark",
                (None, false) => "Dodai · Light",
            },
            family: "dodai",
            description:
                "Frosted glass and gradients: indigo ink, pearl canvas, soft tinted shadows",
            mode,
        },
        mode,
        background: h(c.canvas),
        surface: h(c.surface),
        raised: h(if dark { 0x1b1840 } else { 0xe9e6f8 }),
        border: h(c.lower),
        border_strong: h(c.medium),
        text: h(c.primary),
        muted: h(c.high),
        disabled: h(c.low),
        primary: h(c.primary),
        primary_hover: h(c.primary_hover),
        on_primary: h(c.on_primary),
        secondary: h(if dark { 0xb9a6ff } else { 0x5b3fd6 }),
        on_secondary: h(if dark { c.canvas } else { 0xffffff }),
        accent: h(c.info),
        info: c.info,
        success: c.success,
        warning: c.warning,
        danger: c.error,
        series: [
            c.info,
            c.success,
            c.warning,
            c.error,
            h(c.higher),
            h(0x14b8c4),
            h(0x9b6bff),
            h(c.low),
        ],
        typography: typography(),
        spacing: spacing(),
        shape: shape(),
        glow: 0.0,
        glow_blur: 0.0,
        scanlines: false,
        hard_shadow: 0.0,
    }
    .build();
    t.palette.link = t.palette.text;
    t.motion = motion();
    t.surface_style = surface::style(matches!(mode, Mode::Dark), hue);
    // Glass: the panels are translucent so the canvas mesh (`surface_style.backdrop`) shows
    // through; text keeps its contrast because the mesh stays pale (light) or deep (dark).
    t.palette.surface = t.palette.surface.with_alpha(if dark { 0.66 } else { 0.58 });
    t.palette.surface_raised = t
        .palette
        .surface_raised
        .with_alpha(if dark { 0.72 } else { 0.66 });
    t.shape_system = Some(shape_system());
    t.spec = crate::SpecRef::new(&SPEC);
    t
}

/// The light theme: indigo ink on a pearl canvas.
pub fn light() -> Theme {
    build(Mode::Light)
}

/// The dark theme: lavender-white ink on deep night.
pub fn dark() -> Theme {
    build(Mode::Dark)
}

/// Light and dark, in the signature violet and in each accent.
pub fn variants() -> Vec<Theme> {
    let mut v = vec![light(), dark()];
    for a in &ACCENTS {
        v.push(build_hue(Mode::Light, Some(a)));
        v.push(build_hue(Mode::Dark, Some(a)));
    }
    v
}
