//! Pure token → Freya-attribute mapping. No Freya types, no window: everything
//! here returns strings / numbers that the components feed into `rsx!`.
//!
//! Nothing in this module looks at a theme id or name; shape, glow, hard shadow
//! and label casing are all read from [`Theme`] tokens.

use dodai_core::{ButtonVariant, Color, ControlState, ShapeKind, Theme, Tone};

/// Token colour → Freya colour string. Alpha is always emitted as `0..=255`
/// (Freya reads `0..1` floats ambiguously for values like `1`).
pub fn css(c: Color) -> String {
    if c.a == 255 {
        format!("rgb({}, {}, {})", c.r, c.g, c.b)
    } else {
        format!("rgb({}, {}, {}, {})", c.r, c.g, c.b, c.a)
    }
}

/// Everything needed to paint one themed box.
#[derive(Debug, Clone, PartialEq)]
pub struct ShapeStyle {
    pub kind: ShapeKind,
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: f32,
    pub radius: f32,
    /// Glow colour + blur radius, already scaled by `glow_intensity`.
    pub glow: Option<(Color, f32)>,
    /// Hard (offset, no blur) shadow: offset + colour.
    pub hard_shadow: Option<(f32, Color)>,
}

impl ShapeStyle {
    /// Build from tokens. `glow`/`shadow` switch the respective effect on for
    /// this state; the theme decides whether it is non-zero at all.
    pub fn from_tokens(
        theme: &Theme,
        fill: Option<Color>,
        stroke: Option<Color>,
        glow: bool,
        shadow: bool,
    ) -> Self {
        let fx = &theme.effects;
        Self {
            kind: theme.shape.kind(),
            fill,
            stroke,
            stroke_width: theme.shape.border_width,
            radius: theme.shape.radius,
            glow: (glow && fx.glow_enabled()).then(|| (fx.glow_color(), fx.glow_radius())),
            hard_shadow: (shadow && fx.hard_shadow > 0.0)
                .then(|| (fx.hard_shadow, theme.palette.background.with_alpha(0.7))),
        }
    }

    pub fn is_chamfered(&self) -> bool {
        self.kind == ShapeKind::Chamfered
    }

    /// Freya `background` value.
    pub fn background(&self) -> String {
        self.fill.map_or_else(|| "transparent".into(), css)
    }

    /// Freya `border` value (rect path only; chamfered boxes stroke the polygon).
    pub fn border(&self) -> String {
        match self.stroke {
            Some(c) if self.stroke_width > 0.0 => {
                format!("{} inner {}", self.stroke_width, css(c))
            }
            _ => "none".into(),
        }
    }

    /// Freya `corner_radius` value. Only `Rounded` themes round.
    pub fn corner_radius(&self) -> String {
        match self.kind {
            ShapeKind::Rounded => format!("{}", self.radius),
            _ => "0".into(),
        }
    }

    /// Freya `shadow` value (rect path): glow + hard shadow, comma separated.
    pub fn shadow(&self) -> String {
        let mut v = Vec::new();
        if let Some((o, c)) = self.hard_shadow {
            v.push(format!("{o} {o} 0 0 {}", css(c)));
        }
        if let Some((c, r)) = self.glow {
            v.push(format!("0 0 {r} 0 {}", css(c)));
        }
        if v.is_empty() {
            "none".into()
        } else {
            v.join(", ")
        }
    }

    /// Padding (px) the SVG needs around the box so glow/shadow are not clipped.
    pub fn svg_pad(&self) -> f32 {
        let g = self.glow.map_or(0.0, |(_, r)| r * 1.5);
        let h = self.hard_shadow.map_or(0.0, |(o, _)| o);
        (g.max(h) + self.stroke_width).ceil()
    }
}

fn hex(c: Color) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

fn opacity(c: Color) -> String {
    format!("{:.3}", c.a as f32 / 255.0)
}

fn points(outline: &[(f32, f32)], dx: f32, dy: f32) -> String {
    outline
        .iter()
        .map(|(x, y)| format!("{:.2},{:.2}", x + dx, y + dy))
        .collect::<Vec<_>>()
        .join(" ")
}

/// SVG document for a chamfered box of `w`×`h`, built from `outline` (so the
/// polygon is exactly [`dodai_core::Shape::outline`]). The canvas is
/// `svg_pad()` larger on each side; place it at `-pad,-pad`.
pub fn chamfer_svg(theme: &Theme, style: &ShapeStyle, w: f32, h: f32) -> String {
    let pad = style.svg_pad();
    let outline = theme.shape.outline(w, h);
    let (cw, ch) = (w + 2.0 * pad, h + 2.0 * pad);
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{cw}\" height=\"{ch}\" viewBox=\"0 0 {cw} {ch}\">"
    );
    if let Some((c, r)) = style.glow {
        s += &format!(
            "<defs><filter id=\"g\" x=\"-50%\" y=\"-50%\" width=\"200%\" height=\"200%\">\
             <feGaussianBlur stdDeviation=\"{:.2}\"/></filter></defs>",
            (r / 2.0).max(0.5)
        );
        s += &format!(
            "<polygon points=\"{}\" fill=\"{}\" fill-opacity=\"{}\" filter=\"url(#g)\"/>",
            points(&outline, pad, pad),
            hex(c),
            opacity(c)
        );
    }
    if let Some((o, c)) = style.hard_shadow {
        s += &format!(
            "<polygon points=\"{}\" fill=\"{}\" fill-opacity=\"{}\"/>",
            points(&outline, pad + o, pad + o),
            hex(c),
            opacity(c)
        );
    }
    let fill = style.fill.map_or("none".to_string(), hex);
    let fo = style.fill.map_or("0".to_string(), opacity);
    let stroke = match style.stroke {
        Some(c) if style.stroke_width > 0.0 => format!(
            " stroke=\"{}\" stroke-opacity=\"{}\" stroke-width=\"{}\" stroke-linejoin=\"miter\"",
            hex(c),
            opacity(c),
            style.stroke_width
        ),
        _ => String::new(),
    };
    s += &format!(
        "<polygon points=\"{}\" fill=\"{fill}\" fill-opacity=\"{fo}\"{stroke}/></svg>",
        points(&outline, pad, pad)
    );
    s
}

// ── component styles ────────────────────────────────────────────────────────

/// Button box + label colour.
pub fn button_style(
    theme: &Theme,
    variant: ButtonVariant,
    state: ControlState,
) -> (ShapeStyle, Color) {
    let b = theme.button(variant, state);
    let lit = matches!(
        state,
        ControlState::Hovered | ControlState::Pressed | ControlState::Focused
    );
    (
        ShapeStyle::from_tokens(
            theme,
            b.background,
            b.border,
            lit && variant != ButtonVariant::Ghost,
            state != ControlState::Disabled && b.background.is_some(),
        ),
        b.foreground,
    )
}

/// Panel box; glows when `focused`.
pub fn panel_style(theme: &Theme, focused: bool) -> ShapeStyle {
    let p = theme.panel(focused);
    ShapeStyle::from_tokens(theme, Some(p.background), Some(p.border), focused, true)
}

/// Text input box (also used to override Freya's `Input` theme).
pub fn input_style(theme: &Theme, focused: bool) -> ShapeStyle {
    let i = theme.input(focused);
    ShapeStyle::from_tokens(theme, Some(i.background), Some(i.border), focused, false)
}

/// Tag box + label colour.
pub fn tag_style(theme: &Theme, tone: Tone, solid: bool) -> (ShapeStyle, Color) {
    let t = theme.tag(tone, solid);
    (
        ShapeStyle::from_tokens(theme, t.background, t.border, false, false),
        t.foreground,
    )
}

/// Tab box (flat: tabs never round/chamfer) + label colour + indicator colour.
pub fn tab_style(theme: &Theme, active: bool, hovered: bool) -> (ShapeStyle, Color, Option<Color>) {
    let t = theme.tab(active, hovered);
    let mut s = ShapeStyle::from_tokens(theme, t.background, None, false, false);
    s.kind = ShapeKind::Square;
    (s, t.foreground, t.indicator)
}

/// Switch track style + thumb colour. Glows when on.
pub fn toggle_style(theme: &Theme, on: bool, state: ControlState) -> (ShapeStyle, Color) {
    let t = theme.toggle(on, state);
    (
        ShapeStyle::from_tokens(
            theme,
            Some(t.track),
            Some(t.border),
            on && state != ControlState::Disabled,
            false,
        ),
        t.thumb,
    )
}

/// Progress track + fill colour.
pub fn progress_style(theme: &Theme, tone: Option<Tone>) -> (ShapeStyle, Color) {
    let p = theme.progress(tone);
    (
        ShapeStyle::from_tokens(theme, Some(p.track), Some(p.border), false, false),
        p.fill,
    )
}

/// List row background, foreground, selection marker.
pub fn list_item_style(
    theme: &Theme,
    selected: bool,
    hovered: bool,
) -> (Option<Color>, Color, Option<Color>) {
    let l = theme.list_item(selected, hovered);
    (l.background, l.foreground, l.marker)
}

// ── typography ──────────────────────────────────────────────────────────────

/// Text roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    Heading,
    Label,
    Muted,
    Mono,
}

/// Resolved text style.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    pub family: &'static str,
    pub size: f32,
    pub color: Color,
    pub uppercase: bool,
    pub letter_spacing: f32,
    pub weight: &'static str,
}

impl TextStyle {
    /// Apply casing to `s` (headings and labels only, never mono).
    pub fn apply(&self, s: &str) -> String {
        if self.uppercase {
            s.to_uppercase()
        } else {
            s.to_string()
        }
    }
}

/// Resolve a role. `size` overrides the default scale step when given.
pub fn text_style(theme: &Theme, role: TextRole, size: Option<f32>) -> TextStyle {
    let ty = &theme.typography;
    let p = &theme.palette;
    let upper = ty.uppercase_labels;
    match role {
        TextRole::Heading => TextStyle {
            family: ty.display,
            size: size.unwrap_or(ty.size_xl),
            color: p.text,
            uppercase: upper,
            letter_spacing: ty.label_tracking,
            weight: "bold",
        },
        TextRole::Label => TextStyle {
            family: ty.body,
            size: size.unwrap_or(ty.size_md),
            color: p.text,
            uppercase: upper,
            letter_spacing: ty.label_tracking,
            weight: "normal",
        },
        TextRole::Muted => TextStyle {
            family: ty.body,
            size: size.unwrap_or(ty.size_sm),
            color: p.text_muted,
            uppercase: false,
            letter_spacing: 0.0,
            weight: "normal",
        },
        TextRole::Mono => TextStyle {
            family: ty.mono,
            size: size.unwrap_or(ty.size_sm),
            color: p.text,
            uppercase: false,
            letter_spacing: 0.0,
            weight: "normal",
        },
    }
}

/// Rough label width (px) for sizing shaped boxes without text measurement.
pub fn estimate_text_width(text: &str, size: f32, tracking: f32) -> f32 {
    text.chars().count() as f32 * (size * 0.62 + tracking)
}

#[cfg(test)]
pub(crate) mod testing {
    //! Test-only rounded light and square/uppercase themes built from `Spec`.
    use dodai_core::themes::Spec;
    use dodai_core::{Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};

    fn base(id: &'static str, shape: Shape, glow: f32, hard: f32, upper: bool) -> Theme {
        Spec {
            meta: ThemeMeta {
                id,
                name: id,
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
                uppercase_labels: upper,
                label_tracking: if upper { 1.5 } else { 0.0 },
            },
            spacing: Spacing {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 20.0,
                xl: 32.0,
            },
            shape,
            glow,
            glow_blur: if glow > 0.0 { 10.0 } else { 0.0 },
            scanlines: false,
            hard_shadow: hard,
        }
        .build()
    }

    pub fn light_rounded() -> Theme {
        base(
            "test-light",
            Shape {
                corners: Corners::NONE,
                radius: 8.0,
                chamfer: 0.0,
                border_width: 1.0,
                control_height: 40.0,
            },
            0.0,
            0.0,
            false,
        )
    }

    pub fn square() -> Theme {
        base(
            "test-square",
            Shape {
                corners: Corners::NONE,
                radius: 0.0,
                chamfer: 0.0,
                border_width: 2.0,
                control_height: 36.0,
            },
            0.0,
            3.0,
            true,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;
    use dodai_core::themes;

    #[test]
    fn css_alpha_is_u8() {
        assert_eq!(css(Color::hex(0x102030)), "rgb(16, 32, 48)");
        assert_eq!(
            css(Color::hex(0x102030).with_alpha(0.5)),
            "rgb(16, 32, 48, 128)"
        );
    }

    #[test]
    fn rounded_theme_uses_radius_no_effects() {
        let t = light_rounded();
        let (s, _) = button_style(&t, ButtonVariant::Primary, ControlState::Hovered);
        assert_eq!(s.kind, ShapeKind::Rounded);
        assert_eq!(s.corner_radius(), "8");
        assert_eq!(s.shadow(), "none");
        assert!(!s.is_chamfered());
    }

    #[test]
    fn square_theme_has_zero_radius_and_hard_shadow() {
        let t = square();
        let (s, _) = button_style(&t, ButtonVariant::Primary, ControlState::Idle);
        assert_eq!(s.kind, ShapeKind::Square);
        assert_eq!(s.corner_radius(), "0");
        assert!(s.shadow().starts_with("3 3 0 0 "));
        let (o, _) = button_style(&t, ButtonVariant::Outline, ControlState::Idle);
        assert_eq!(o.border(), format!("2 inner {}", css(o.stroke.unwrap())));
    }

    #[test]
    fn disabled_button_never_glows_or_shadows() {
        for t in themes::all() {
            let (s, _) = button_style(&t, ButtonVariant::Primary, ControlState::Disabled);
            assert!(s.glow.is_none() && s.hard_shadow.is_none(), "{}", t.id());
        }
    }

    #[test]
    fn glow_follows_effects_intensity() {
        for t in themes::all() {
            let hot = button_style(&t, ButtonVariant::Primary, ControlState::Hovered).0;
            assert_eq!(hot.glow.is_some(), t.effects.glow_enabled(), "{}", t.id());
            let off = t.clone().with_glow_intensity(0.0);
            assert!(
                button_style(&off, ButtonVariant::Primary, ControlState::Hovered)
                    .0
                    .glow
                    .is_none()
            );
            let idle = button_style(&t, ButtonVariant::Primary, ControlState::Idle).0;
            assert!(idle.glow.is_none());
        }
    }

    #[test]
    fn chamfer_svg_matches_outline_per_shape_kind() {
        for t in themes::all().into_iter().chain([light_rounded(), square()]) {
            let (s, _) = button_style(&t, ButtonVariant::Primary, ControlState::Idle);
            assert_eq!(s.kind, t.shape.kind());
            let svg = chamfer_svg(&t, &s, 120.0, 40.0);
            let n_polys = svg.matches("<polygon").count();
            assert!(n_polys >= 1);
            let main = t.shape.outline(120.0, 40.0);
            let pad = s.svg_pad();
            let last = points(&main, pad, pad);
            assert!(svg.contains(&last), "{}", t.id());
            match s.kind {
                ShapeKind::Chamfered => assert!(main.len() > 4),
                _ => assert_eq!(main.len(), 4),
            }
        }
    }

    #[test]
    fn svg_pad_covers_glow() {
        let t = themes::cyberpunk::night_city().with_glow_intensity(2.0);
        let s = button_style(&t, ButtonVariant::Primary, ControlState::Hovered).0;
        if let Some((_, r)) = s.glow {
            assert!(s.svg_pad() >= r);
        }
    }

    #[test]
    fn uppercase_follows_token_and_mono_never() {
        let sq = square();
        let lr = light_rounded();
        assert_eq!(text_style(&sq, TextRole::Label, None).apply("Hack"), "HACK");
        assert_eq!(text_style(&lr, TextRole::Label, None).apply("Hack"), "Hack");
        assert_eq!(text_style(&sq, TextRole::Heading, None).apply("a"), "A");
        assert_eq!(text_style(&sq, TextRole::Mono, None).apply("a"), "a");
        assert_eq!(text_style(&sq, TextRole::Muted, None).apply("a"), "a");
        assert_eq!(text_style(&sq, TextRole::Label, None).letter_spacing, 1.5);
        assert_eq!(text_style(&lr, TextRole::Mono, Some(20.0)).size, 20.0);
    }

    #[test]
    fn panel_focus_glows_only_when_theme_has_glow() {
        for t in themes::all().into_iter().chain([light_rounded(), square()]) {
            let p = panel_style(&t, true);
            assert_eq!(p.glow.is_some(), t.effects.glow_enabled(), "{}", t.id());
            assert!(panel_style(&t, false).glow.is_none());
        }
    }

    #[test]
    fn component_styles_resolve_everywhere() {
        for t in themes::all().into_iter().chain([light_rounded(), square()]) {
            let (tag, fg) = tag_style(&t, Tone::Danger, true);
            assert_eq!(tag.fill, Some(t.palette.danger));
            assert_eq!(fg, t.on(t.palette.danger));
            let (tab, _, ind) = tab_style(&t, true, false);
            assert_eq!(tab.kind, ShapeKind::Square);
            assert!(ind.is_some());
            let (on, thumb_on) = toggle_style(&t, true, ControlState::Idle);
            let (off, _) = toggle_style(&t, false, ControlState::Idle);
            assert_ne!(on.fill, off.fill);
            assert_eq!(thumb_on, t.palette.on_primary);
            let (_, fill) = progress_style(&t, None);
            assert_eq!(fill, t.palette.primary);
            assert!(list_item_style(&t, true, false).2.is_some());
            assert!(input_style(&t, false).fill.is_some());
        }
    }
}
