//! **Aurora**: frosted glass and gradients. Deep indigo ink on a pearl canvas (or lavender-white
//! on deep night), 16 px controls, 30 px containers, hairline borders, tinted soft shadows and a
//! violet-through-blue-to-teal gradient as the signature accent. Token values are Aurora's own
//! (see [`tokens`](crate::aurora::tokens)).

use crate::themes::ThemeBuilder;
use crate::{Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};

mod spec;
pub mod surface;
pub mod tokens;
pub use spec::{shape_presets, shape_system, shape_system_compact, Aurora, SHAPE_SCALE, SPEC};

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
        primary: rgb(t.primary.color()),
        // Hover: the primary eased 22 % towards the canvas.
        primary_hover: rgb(t.primary.color().mix(canvas, 0.22)),
        on_primary: rgb(canvas),
        higher: over(t.contrast_higher),
        high: over(t.contrast_high),
        medium: over(t.contrast_medium),
        low: over(t.contrast_low),
        lower: over(t.contrast_lower),
        info: rgb(t.info.base.color()),
        success: rgb(t.success.base.color()),
        warning: rgb(t.warning.base.color()),
        error: rgb(t.error.base.color()),
    }
}

fn build(mode: Mode) -> Theme {
    let dark = mode == Mode::Dark;
    let c = &ink(if dark { &tokens::DARK } else { &tokens::LIGHT });
    let mut t = ThemeBuilder {
        meta: ThemeMeta {
            id: if dark { "aurora-dark" } else { "aurora" },
            name: if dark {
                "Aurora · Dark"
            } else {
                "Aurora · Light"
            },
            family: "aurora",
            description:
                "Frosted glass and gradients: indigo ink, pearl canvas, soft tinted shadows",
            mode,
        },
        mode,
        background: c.canvas,
        surface: c.surface,
        raised: if dark { 0x1b1840 } else { 0xe9e6f8 },
        border: c.lower,
        border_strong: c.medium,
        text: c.primary,
        muted: c.high,
        disabled: c.low,
        primary: c.primary,
        primary_hover: c.primary_hover,
        on_primary: c.on_primary,
        secondary: if dark { 0xb9a6ff } else { 0x5b3fd6 },
        on_secondary: if dark { c.canvas } else { 0xffffff },
        accent: c.info,
        info: c.info,
        success: c.success,
        warning: c.warning,
        danger: c.error,
        series: [
            c.info, c.success, c.warning, c.error, c.higher, 0x14b8c4, 0x9b6bff, c.low,
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
    t.surface_style = surface::style(matches!(mode, Mode::Dark));
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

/// Both themes.
pub fn variants() -> Vec<Theme> {
    vec![light(), dark()]
}
