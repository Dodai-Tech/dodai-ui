//! Pure token → paint mapping. No Freya types, no window: everything here returns
//! plain numbers and [`Color`]s that [`crate::paint`] turns into Freya values.
//!
//! Nothing in this module looks at a theme id or name; shape, glow, hard shadow
//! and label casing are all read from [`Theme`] tokens.

use dodai_core::{
    ButtonVariant, Color, ControlState, ShapeKind, Surface, SurfaceClass, SurfaceFx, Theme, Tone,
};
use std::sync::RwLock;

static SURFACE_FX: RwLock<Option<SurfaceFx>> = RwLock::new(None);

/// The app-wide surface mix: containers and input fields it frosts become glass.
pub fn surface_fx() -> SurfaceFx {
    SURFACE_FX
        .read()
        .map_or_else(|e| *e.into_inner(), |g| *g)
        .unwrap_or(SurfaceFx::NONE)
}

/// The surface mix of `theme`'s surfaces: the app's, or until it sets one the design's own glass
/// ([`Theme::default_surface`]).
pub fn surface_fx_for(theme: &Theme) -> SurfaceFx {
    let set = SURFACE_FX.read().map_or_else(|e| *e.into_inner(), |g| *g);
    set.unwrap_or_else(|| theme.default_surface())
}

/// Set the app-wide surface mix (`frost` reaches containers and input fields). Applies to
/// everything built afterwards; Freya blurs what is behind a pane natively.
pub fn set_surface_fx(fx: SurfaceFx) {
    *SURFACE_FX.write().unwrap_or_else(|e| e.into_inner()) = Some(fx);
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
    /// Blur radius of the backdrop behind a frosted pane, px (glass, see [`Self::with_glass`]).
    pub blur: Option<f32>,
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
            blur: None,
        }
    }

    /// The glass the surface mix gives a `class` surface: the fill turns see-through and, when
    /// frosted, the backdrop behind it blurs. Unchanged without the mix.
    pub fn with_glass(mut self, theme: &Theme, class: SurfaceClass) -> Self {
        let glass = dodai_core::surface_glass(theme, class, &Surface::INHERIT, surface_fx_for(theme));
        if glass.is_solid() {
            return self;
        }
        self.fill = self.fill.map(|c| glass.color(c));
        self.blur = glass.frosted().then_some(glass.blur * glass.amount);
        self
    }

    pub fn is_chamfered(&self) -> bool {
        self.kind == ShapeKind::Chamfered
    }

    /// Corner radius in px. Only `Rounded` themes round.
    pub fn corner_radius(&self) -> f32 {
        match self.kind {
            ShapeKind::Rounded => self.radius,
            _ => 0.0,
        }
    }

    /// Border (colour, width) for the rect path; chamfered boxes stroke the polygon.
    pub fn border(&self) -> Option<(Color, f32)> {
        self.stroke
            .filter(|_| self.stroke_width > 0.0)
            .map(|c| (c, self.stroke_width))
    }

    /// The single shadow Freya can paint on a rect: the hard (offset) shadow when the
    /// theme has one, otherwise the glow. `(x, y, blur, colour)`.
    pub fn shadow(&self) -> Option<(f32, f32, f32, Color)> {
        if let Some((o, c)) = self.hard_shadow {
            return Some((o, o, 0.0, c));
        }
        self.glow.map(|(c, r)| (0.0, 0.0, r, c))
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
        .with_glass(theme, SurfaceClass::Panel)
}

/// Text input box (also used to override Freya's `Input` theme).
pub fn input_style(theme: &Theme, focused: bool) -> ShapeStyle {
    let i = theme.input(focused);
    ShapeStyle::from_tokens(theme, Some(i.background), Some(i.border), focused, false)
        .with_glass(theme, SurfaceClass::Input)
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
    let t = theme.tab(dodai_core::ItemState::of(active, hovered));
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
    let l = theme.list_item(dodai_core::ItemState::of(selected, hovered));
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
    /// CSS-style weight: 400 regular, 700 bold.
    pub weight: u16,
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
            weight: 700,
        },
        TextRole::Label => TextStyle {
            family: ty.body,
            size: size.unwrap_or(ty.size_md),
            color: p.text,
            uppercase: upper,
            letter_spacing: ty.label_tracking,
            weight: 400,
        },
        TextRole::Muted => TextStyle {
            family: ty.body,
            size: size.unwrap_or(ty.size_sm),
            color: p.text_muted,
            uppercase: false,
            letter_spacing: 0.0,
            weight: 400,
        },
        TextRole::Mono => TextStyle {
            family: ty.mono,
            size: size.unwrap_or(ty.size_sm),
            color: p.text,
            uppercase: false,
            letter_spacing: 0.0,
            weight: 400,
        },
    }
}

/// Rough label width (px) for sizing shaped boxes without text measurement.
pub fn estimate_text_width(text: &str, size: f32, tracking: f32) -> f32 {
    text.chars().count() as f32 * (size * 0.62 + tracking)
}

#[cfg(test)]
pub(crate) mod testing {
    //! Test-only rounded light and square/uppercase themes built from `ThemeBuilder`.
    use dodai_core::themes::ThemeBuilder;
    use dodai_core::{Corners, Mode, Shape, Theme, ThemeMeta, Typography};

    fn base(id: &'static str, shape: Shape, glow: f32, hard: f32, upper: bool) -> Theme {
        ThemeBuilder {
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
            shape,
            glow,
            glow_blur: if glow > 0.0 { 10.0 } else { 0.0 },
            hard_shadow: hard,
            ..ThemeBuilder::neutral(ThemeMeta {
                id,
                name: id,
                family: "test",
                description: "",
                mode: Mode::Light,
            })
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
                scoop: false,
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
                scoop: false,
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
    fn rounded_theme_uses_radius_no_effects() {
        let t = light_rounded();
        let (s, _) = button_style(&t, ButtonVariant::Primary, ControlState::Hovered);
        assert_eq!(s.kind, ShapeKind::Rounded);
        assert_eq!(s.corner_radius(), 8.0);
        assert_eq!(s.shadow(), None);
        assert!(!s.is_chamfered());
    }

    #[test]
    fn square_theme_has_zero_radius_and_hard_shadow() {
        let t = square();
        let (s, _) = button_style(&t, ButtonVariant::Primary, ControlState::Idle);
        assert_eq!(s.kind, ShapeKind::Square);
        assert_eq!(s.corner_radius(), 0.0);
        assert_eq!(
            s.shadow().map(|(x, y, b, _)| (x, y, b)),
            Some((3.0, 3.0, 0.0))
        );
        let (o, _) = button_style(&t, ButtonVariant::Outline, ControlState::Idle);
        assert_eq!(o.border(), Some((o.stroke.unwrap(), 2.0)));
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
        let t = themes::cyberpunk::yellow().with_glow_intensity(2.0);
        let s = button_style(&t, ButtonVariant::Primary, ControlState::Hovered).0;
        if let Some((_, r)) = s.glow {
            assert!(s.svg_pad() >= r);
        }
    }

    #[test]
    fn the_surface_mix_turns_panels_and_inputs_into_glass_but_not_buttons() {
        let t = light_rounded();
        set_surface_fx(SurfaceFx::NONE);
        assert!(panel_style(&t, false).blur.is_none());
        set_surface_fx(SurfaceFx {
            frost: 1.0,
            targets: dodai_core::SurfaceTargets::ALL,
            ..SurfaceFx::NONE
        });
        let panel = panel_style(&t, false);
        assert!(panel.blur.is_some_and(|b| b > 0.0));
        assert!(panel
            .fill
            .is_some_and(|f| f.a < t.panel(false).background.a));
        assert!(input_style(&t, false).blur.is_some());
        let (button, _) = button_style(&t, ButtonVariant::Primary, ControlState::Idle);
        assert!(button.blur.is_none());
        set_surface_fx(SurfaceFx::NONE);
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
        for t in themes::all().into_iter().chain([square()]) {
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
            assert_eq!(
                list_item_style(&t, true, false).2.is_some(),
                t.spec.name() == "Cyberpunk"
            );
            assert!(input_style(&t, false).fill.is_some());
        }
    }
}
