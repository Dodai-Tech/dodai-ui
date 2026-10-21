//! Plain-data snapshot of a [`Theme`]: everything the Slint `Theme` global
//! needs, resolved from tokens only (no theme-id branching, no Slint types so
//! it is testable without a window).

use dodai_core::{ButtonVariant, ControlState, Shape, ShapeKind, Theme, Tone};

/// RGBA colour, 0..=255 per channel.
pub type Rgba = [u8; 4];

/// Fully transparent (used for "no background / no border").
pub const TRANSPARENT: Rgba = [0, 0, 0, 0];

/// Button variants in index order (`variant * 4 + state`).
pub const BUTTON_VARIANTS: [ButtonVariant; 7] = [
    ButtonVariant::Primary,
    ButtonVariant::Secondary,
    ButtonVariant::Outline,
    ButtonVariant::OutlineSecondary,
    ButtonVariant::Danger,
    ButtonVariant::Ghost,
    ButtonVariant::Scrim,
];
/// Interaction states in index order: idle, hovered, pressed, disabled.
pub const STATES: [ControlState; 4] = [
    ControlState::Idle,
    ControlState::Hovered,
    ControlState::Pressed,
    ControlState::Disabled,
];
/// Status tones in index order (`tone * 2 + solid`).
pub const TONES: [Tone; 4] = [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger];

fn c(col: dodai_core::Color) -> Rgba {
    [col.r, col.g, col.b, col.a]
}
fn o(col: Option<dodai_core::Color>) -> Rgba {
    col.map_or(TRANSPARENT, c)
}

/// Everything pushed into the Slint `Theme` global.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct ThemeValues {
    pub background: Rgba,
    pub surface: Rgba,
    pub surface_raised: Rgba,
    pub overlay: Rgba,
    pub border: Rgba,
    pub border_strong: Rgba,
    pub text: Rgba,
    pub text_muted: Rgba,
    pub text_disabled: Rgba,
    pub primary: Rgba,
    pub on_primary: Rgba,
    pub secondary: Rgba,
    pub danger: Rgba,
    pub selection: Rgba,
    pub focus_ring: Rgba,
    /// Glow colour with intensity already applied to alpha.
    pub glow: Rgba,
    pub radius: f32,
    pub chamfer: f32,
    pub border_width: f32,
    pub control_height: f32,
    pub size_xs: f32,
    pub size_sm: f32,
    pub size_md: f32,
    pub size_lg: f32,
    pub size_xl: f32,
    pub size_xxl: f32,
    pub sp_xs: f32,
    pub sp_sm: f32,
    pub sp_md: f32,
    pub sp_lg: f32,
    pub sp_xl: f32,
    pub glow_radius: f32,
    pub hard_shadow: f32,
    pub label_tracking: f32,
    /// True for [`ShapeKind::Chamfered`].
    pub chamfered: bool,
    pub cut_tl: bool,
    pub cut_tr: bool,
    pub cut_br: bool,
    pub cut_bl: bool,
    pub glow_enabled: bool,
    pub scanlines: bool,
    pub uppercase: bool,
    pub font_display: String,
    pub font_body: String,
    pub font_mono: String,
    /// `variant * 4 + state`.
    pub button_bg: Vec<Rgba>,
    pub button_fg: Vec<Rgba>,
    pub button_border: Vec<Rgba>,
    /// `tone * 2 + solid`.
    pub tag_bg: Vec<Rgba>,
    pub tag_fg: Vec<Rgba>,
    pub tag_border: Vec<Rgba>,
    /// `active * 2 + hovered`.
    pub tab_bg: Vec<Rgba>,
    pub tab_fg: Vec<Rgba>,
    pub tab_indicator: Vec<Rgba>,
    /// `on * 4 + state`.
    pub toggle_track: Vec<Rgba>,
    pub toggle_thumb: Vec<Rgba>,
    pub toggle_border: Vec<Rgba>,
    /// Index 0 = brand, 1.. = tones.
    pub progress_track: Vec<Rgba>,
    pub progress_fill: Vec<Rgba>,
    pub progress_border: Vec<Rgba>,
    /// `selected * 2 + hovered`.
    pub list_bg: Vec<Rgba>,
    pub list_fg: Vec<Rgba>,
    pub list_marker: Vec<Rgba>,
    /// `focused`.
    pub panel_bg: Vec<Rgba>,
    pub panel_border: Vec<Rgba>,
    pub panel_title: Vec<Rgba>,
    pub input_bg: Vec<Rgba>,
    pub input_border: Vec<Rgba>,
    pub input_text: Rgba,
    pub input_placeholder: Rgba,
}

impl ThemeValues {
    /// Resolve every Slint-facing value from `t`.
    pub fn from_theme(t: &Theme) -> Self {
        let p = &t.palette;
        let (ty, sp, sh, fx) = (&t.typography, &t.spacing, &t.shape, &t.effects);
        let mut v = Self {
            background: c(p.background),
            surface: c(p.surface),
            surface_raised: c(p.surface_raised),
            overlay: c(p.overlay),
            border: c(p.border),
            border_strong: c(p.border_strong),
            text: c(p.text),
            text_muted: c(p.text_muted),
            text_disabled: c(p.text_disabled),
            primary: c(p.primary),
            on_primary: c(p.on_primary),
            secondary: c(p.secondary),
            danger: c(p.danger),
            selection: c(p.selection),
            focus_ring: c(p.focus_ring),
            glow: c(fx.glow_color()),
            radius: sh.radius,
            chamfer: sh.chamfer,
            border_width: sh.border_width,
            control_height: sh.control_height,
            size_xs: ty.size_xs,
            size_sm: ty.size_sm,
            size_md: ty.size_md,
            size_lg: ty.size_lg,
            size_xl: ty.size_xl,
            size_xxl: ty.size_xxl,
            sp_xs: sp.xs,
            sp_sm: sp.sm,
            sp_md: sp.md,
            sp_lg: sp.lg,
            sp_xl: sp.xl,
            glow_radius: fx.glow_radius(),
            hard_shadow: fx.hard_shadow,
            label_tracking: ty.label_tracking,
            chamfered: sh.kind() == ShapeKind::Chamfered,
            cut_tl: sh.corners.top_left,
            cut_tr: sh.corners.top_right,
            cut_br: sh.corners.bottom_right,
            cut_bl: sh.corners.bottom_left,
            glow_enabled: fx.glow_enabled(),
            scanlines: fx.scanlines,
            uppercase: ty.uppercase_labels,
            font_display: first_family(ty.display),
            font_body: first_family(ty.body),
            font_mono: first_family(ty.mono),
            button_bg: vec![],
            button_fg: vec![],
            button_border: vec![],
            tag_bg: vec![],
            tag_fg: vec![],
            tag_border: vec![],
            tab_bg: vec![],
            tab_fg: vec![],
            tab_indicator: vec![],
            toggle_track: vec![],
            toggle_thumb: vec![],
            toggle_border: vec![],
            progress_track: vec![],
            progress_fill: vec![],
            progress_border: vec![],
            list_bg: vec![],
            list_fg: vec![],
            list_marker: vec![],
            panel_bg: vec![],
            panel_border: vec![],
            panel_title: vec![],
            input_bg: vec![],
            input_border: vec![],
            input_text: c(t.input(false).text),
            input_placeholder: c(t.input(false).placeholder),
        };
        for var in BUTTON_VARIANTS {
            for st in STATES {
                let b = t.button(var, st);
                v.button_bg.push(o(b.background));
                v.button_fg.push(c(b.foreground));
                v.button_border.push(o(b.border));
            }
        }
        for tone in TONES {
            for solid in [false, true] {
                let g = t.tag(tone, solid);
                v.tag_bg.push(o(g.background));
                v.tag_fg.push(c(g.foreground));
                v.tag_border.push(o(g.border));
            }
        }
        for active in [false, true] {
            for hov in [false, true] {
                let g = t.tab(active, hov);
                v.tab_bg.push(o(g.background));
                v.tab_fg.push(c(g.foreground));
                v.tab_indicator.push(o(g.indicator));
            }
        }
        for on in [false, true] {
            for st in STATES {
                let g = t.toggle(on, st);
                v.toggle_track.push(c(g.track));
                v.toggle_thumb.push(c(g.thumb));
                v.toggle_border.push(c(g.border));
            }
        }
        for tone in std::iter::once(None).chain(TONES.into_iter().map(Some)) {
            let g = t.progress(tone);
            v.progress_track.push(c(g.track));
            v.progress_fill.push(c(g.fill));
            v.progress_border.push(c(g.border));
        }
        for sel in [false, true] {
            for hov in [false, true] {
                let g = t.list_item(sel, hov);
                v.list_bg.push(o(g.background));
                v.list_fg.push(c(g.foreground));
                v.list_marker.push(o(g.marker));
            }
        }
        for f in [false, true] {
            let g = t.panel(f);
            v.panel_bg.push(c(g.background));
            v.panel_border.push(c(g.border));
            v.panel_title.push(c(g.title));
            let i = t.input(f);
            v.input_bg.push(c(i.background));
            v.input_border.push(c(i.border));
        }
        v
    }
}

/// First family of a comma-separated fallback list, unquoted.
pub fn first_family(list: &str) -> String {
    list.split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(|ch| ch == '"' || ch == '\'')
        .to_string()
}

/// Outline polygon for a `w`×`h` control, from [`Shape::outline`]; handy for
/// custom Slint `Path`s built outside the bundled components.
pub fn outline(shape: &Shape, w: f32, h: f32) -> Vec<(f32, f32)> {
    shape.outline(w, h)
}
