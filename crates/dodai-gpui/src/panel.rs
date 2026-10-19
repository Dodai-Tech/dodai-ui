//! [`Panel`] (card), [`InputBox`] (text-input container) and [`DodaiTooltip`].

use crate::paint::{shaped, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, px, AnyElement, AnyView, App, AppContext as _, Context, Div, IntoElement, ParentElement,
    Render, RenderOnce, SharedString, Styled, Window,
};

/// Border of a section/panel: thickness and strength. Reusable value type;
/// set it per panel with [`Panel::border`] or for every panel with
/// [`set_section_border`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderStyle {
    /// Line thickness in px (`0` = no border).
    pub width: f32,
    /// Colour strength `0..=1` (multiplies the theme border's alpha).
    pub opacity: f32,
    /// Corner that carries the accent (theme primary, e.g. the yellow line); the
    /// other three edges keep the grey border. `None` = plain border all round.
    pub accent: Option<Corner>,
    /// Thickness of the accent line in px.
    pub accent_width: f32,
}

/// A corner of a box (where the accent bracket sits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    pub const ALL: [Corner; 4] = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ];
    /// The diagonally opposite corner (where the grey bracket goes).
    pub fn opposite(self) -> Corner {
        match self {
            Corner::TopLeft => Corner::BottomRight,
            Corner::BottomRight => Corner::TopLeft,
            Corner::TopRight => Corner::BottomLeft,
            Corner::BottomLeft => Corner::TopRight,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Corner::TopLeft => "Top left",
            Corner::TopRight => "Top right",
            Corner::BottomLeft => "Bottom left",
            Corner::BottomRight => "Bottom right",
        }
    }
}

impl BorderStyle {
    /// Barely-there hairline.
    pub const SUBTLE: BorderStyle = BorderStyle {
        width: 0.75,
        opacity: 0.45,
        accent: None,
        accent_width: 2.0,
    };
    /// Default for sections: thin and soft.
    pub const THIN: BorderStyle = BorderStyle {
        width: 1.0,
        opacity: 0.6,
        accent: None,
        accent_width: 2.0,
    };
    /// The theme's own border, unchanged strength.
    pub const NORMAL: BorderStyle = BorderStyle {
        width: 1.5,
        opacity: 1.0,
        accent: None,
        accent_width: 2.0,
    };
    /// Heavy frame.
    pub const BOLD: BorderStyle = BorderStyle {
        width: 3.0,
        opacity: 1.0,
        accent: None,
        accent_width: 2.0,
    };
    pub const NONE: BorderStyle = BorderStyle {
        width: 0.0,
        opacity: 0.0,
        accent: None,
        accent_width: 2.0,
    };

    pub fn new(width: f32, opacity: f32) -> Self {
        Self {
            width,
            opacity,
            accent: None,
            accent_width: 2.0,
        }
        .clamped()
    }
    /// Same border with the accent line on `edge` (or none).
    pub fn accent(mut self, edge: Option<Corner>) -> Self {
        self.accent = edge;
        self
    }
    /// Same border with an accent line of `width` px.
    pub fn accent_width(mut self, width: f32) -> Self {
        self.accent_width = width;
        self.clamped()
    }
    fn clamped(self) -> Self {
        Self {
            width: self.width.clamp(0.0, 8.0),
            opacity: self.opacity.clamp(0.0, 1.0),
            accent: self.accent,
            accent_width: self.accent_width.clamp(0.5, 8.0),
        }
    }
    /// Apply to a paint style (pure).
    pub fn apply(self, mut s: ShapeStyle) -> ShapeStyle {
        let b = self.clamped();
        s.stroke = match s.stroke {
            Some(c) if b.width > 0.0 && b.opacity > 0.0 => Some(gpui::Hsla {
                a: c.a * b.opacity,
                ..c
            }),
            _ => None,
        };
        s.stroke_width = b.width;
        s
    }
}

impl Default for BorderStyle {
    fn default() -> Self {
        Self::THIN
    }
}

struct SectionBorderGlobal(BorderStyle);
impl gpui::Global for SectionBorderGlobal {}

/// Border used by every [`Panel`] without its own [`Panel::border`].
pub fn section_border(cx: &App) -> BorderStyle {
    cx.try_global::<SectionBorderGlobal>()
        .map(|g| g.0)
        .unwrap_or_default()
}

/// Set the border of every panel and repaint.
pub fn set_section_border(cx: &mut App, b: BorderStyle) {
    cx.set_global(SectionBorderGlobal(b.clamped()));
    cx.refresh_windows();
}

/// Paint style of a panel (pure).
pub fn panel_style(theme: &Theme, focused: bool) -> ShapeStyle {
    let p = theme.panel(focused);
    ShapeStyle::from_tokens(theme, Some(p.background), Some(p.border), focused, true)
}

/// Paint style of an input box (pure).
pub fn input_style(theme: &Theme, focused: bool) -> ShapeStyle {
    let i = theme.input(focused);
    ShapeStyle::from_tokens(theme, Some(i.background), Some(i.border), focused, false)
}

/// Shaped card with optional title; focused border/glow; hard shadow from tokens.
#[derive(IntoElement)]
pub struct Panel {
    title: Option<SharedString>,
    focused: bool,
    subheader: Option<AnyElement>,
    scope: Option<crate::motion::EntrySet>,
    scope_key: u32,
    children: Vec<AnyElement>,
    refine: Option<Box<dyn FnOnce(gpui::Div) -> gpui::Div>>,
    corners: Option<dodai_core::Corners>,
    border: Option<BorderStyle>,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            border: None,
            title: None,
            corners: None,
            focused: false,
            subheader: None,
            scope: None,
            scope_key: 0,
            children: Vec::new(),
            refine: None,
        }
    }
    /// Title shown above the content (uppercased per theme).
    /// Entry effects for the components *inside* the panel (the panel itself
    /// keeps the app-wide motion).
    pub fn scope(mut self, set: Option<crate::motion::EntrySet>) -> Self {
        self.scope = set;
        self
    }
    /// Changing this restarts the inner components' entry animations (the
    /// panel itself is untouched, so it does not flicker).
    pub fn scope_key(mut self, k: u32) -> Self {
        self.scope_key = k;
        self
    }
    /// Element shown directly below the title (e.g. a toolbar or chips).
    pub fn subheader(mut self, el: impl IntoElement) -> Self {
        self.subheader = Some(el.into_any_element());
        self
    }
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
    /// Border thickness/strength for this panel (default: [`section_border`]).
    pub fn border(mut self, b: BorderStyle) -> Self {
        self.border = Some(b);
        self
    }
    /// Highlight border (and glow when enabled).
    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }
    /// Extra layout styling on the root div (width, margin…).
    pub fn refine(mut self, f: impl FnOnce(gpui::Div) -> gpui::Div + 'static) -> Self {
        self.refine = Some(Box::new(f));
        self
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Panel {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

/// L-bracket of `w` px in `corner`, arms pulled `inset` px off a cut or
/// rounded corner.
fn accent_mark(corner: Corner, c: gpui::Hsla, w: f32, inset: f32) -> gpui::Div {
    const ARM: f32 = 28.0;
    let top = matches!(corner, Corner::TopLeft | Corner::TopRight);
    let left = matches!(corner, Corner::TopLeft | Corner::BottomLeft);
    let h = div().absolute().bg(c).w(px(ARM)).h(px(w));
    let v = div().absolute().bg(c).h(px(ARM)).w(px(w));
    let (h, v) = if top {
        (h.top_0(), v.top(px(inset)))
    } else {
        (h.bottom_0(), v.bottom(px(inset)))
    };
    let (h, v) = if left {
        (h.left(px(inset)), v.left_0())
    } else {
        (h.right(px(inset)), v.right_0())
    };
    div().absolute().size_full().child(h).child(v)
}

/// How far a corner pulls the bracket in: the radius when that corner is
/// rounded or cut, else 0.
fn corner_inset(shape: &dodai_core::Shape, corner: Corner) -> f32 {
    let cut = match corner {
        Corner::TopLeft => shape.corners.top_left,
        Corner::TopRight => shape.corners.top_right,
        Corner::BottomLeft => shape.corners.bottom_left,
        Corner::BottomRight => shape.corners.bottom_right,
    };
    match shape.kind() {
        dodai_core::ShapeKind::Rounded => shape.radius,
        dodai_core::ShapeKind::Chamfered if cut => shape.radius,
        _ => 0.0,
    }
}

impl Panel {
    fn build(self, _: &mut Window, cx: &mut App) -> gpui::Div {
        let t = theme_of(cx);
        let p = t.panel(self.focused);
        let mut shape = t.shape.clone();
        if let Some(c) = self.corners {
            shape.corners = c;
        }
        let border = self.border.unwrap_or_else(|| section_border(cx));
        let mut style = border.apply(panel_style(&t, self.focused));
        let grey = style.stroke.map(|_| gpui::Hsla {
            a: hsla(t.palette.border_strong).a * border.opacity.clamp(0.0, 1.0),
            ..hsla(t.palette.border_strong)
        });
        if border.accent.is_some() {
            // Accent mode replaces the full frame: yellow here, grey opposite.
            style.stroke = None;
            style.shadow = None;
        }
        let mut d = shaped(&shape, style);
        if let Some(edge) = border.accent {
            let b = border.clamped();
            let accent = hsla(t.panel(true).border);
            if let Some(g) = grey {
                let o = edge.opposite();
                d = d.child(accent_mark(o, g, b.width, corner_inset(&shape, o)));
            }
            d = d.child(accent_mark(
                edge,
                accent,
                b.accent_width,
                corner_inset(&shape, edge),
            ));
        }
        // Absolute accent marks are positioned against the padding box in gpui,
        // so with an accent the padding moves to an inner wrapper and the marks
        // sit on the true outer edge.
        let wrap = border.accent.is_some();
        let mut d = d
            .flex()
            .flex_col()
            .min_w_0()
            .text_color(hsla(t.palette.text))
            .text_size(px(t.typography.size_md))
            .font_family(first_family(t.typography.body));
        let mut items: Vec<AnyElement> = Vec::new();
        if let Some(title) = self.title {
            items.push(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(p.title))
                    .child(format_label(&t.typography, &title))
                    .into_any_element(),
            );
        }
        if let Some(sh) = self.subheader {
            items.push(sh);
        }
        match self.scope {
            Some(set) => {
                let key = self.scope_key.wrapping_mul(256);
                items.extend(self.children.into_iter().enumerate().map(|(i, c)| {
                    crate::motion::EntryScope::new(set, c)
                        .delay(std::time::Duration::from_millis(
                            set.stagger_ms() as u64 * i as u64,
                        ))
                        .key(("pscope", key.wrapping_add(i as u32)))
                        .into_any_element()
                }));
            }
            None => items.extend(self.children),
        }
        let pad = |x: Div| {
            x.flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .p(px(t.spacing.lg))
        };
        d = if wrap {
            d.child(pad(div()).min_w_0().children(items))
        } else {
            pad(d).children(items)
        };
        match self.refine {
            Some(f) => f(d),
            None => d,
        }
    }
}

impl RenderOnce for Panel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry_div(self.build(window, cx), "Panel", motion)
    }
}

/// Styled container for a text-input entity. gpui 0.2.2 ships no text input;
/// put your own editor view/entity inside (or use `.placeholder`).
#[derive(IntoElement)]
pub struct InputBox {
    focused: bool,
    placeholder: Option<SharedString>,
    children: Vec<AnyElement>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(InputBox);

impl InputBox {
    pub fn new() -> Self {
        Self {
            corners: None,
            focused: false,
            placeholder: None,
            children: Vec::new(),
        }
    }
    /// Focus ring: border colour change and glow.
    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }
    /// Shown (dimmed) while there are no children.
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = Some(p.into());
        self
    }
}

impl Default for InputBox {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for InputBox {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl InputBox {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let i = t.input(self.focused);
        let d = shaped(&t.shape, input_style(&t, self.focused))
            .h(px(t.shape.control_height))
            .px(px(t.spacing.sm))
            .flex()
            .items_center()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(i.text));
        if self.children.is_empty() {
            d.child(
                div()
                    .text_color(hsla(i.placeholder))
                    .child(self.placeholder.unwrap_or_default()),
            )
        } else {
            d.children(self.children)
        }
    }
}

crate::entry_render!(InputBox);

/// Tooltip view: `.tooltip(DodaiTooltip::text("Hint"))` on an id'd element.
pub struct DodaiTooltip {
    text: SharedString,
}

impl DodaiTooltip {
    /// Builder for gpui's `.tooltip(...)`.
    pub fn text(
        text: impl Into<SharedString>,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        let text: SharedString = text.into();
        move |_, cx| {
            let text = text.clone();
            cx.new(|_| DodaiTooltip { text }).into()
        }
    }
}

impl Render for DodaiTooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme_of(cx);
        let style = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.overlay),
            Some(t.palette.border_strong),
            false,
            false,
        );
        shaped(&t.shape, style)
            .px(px(t.spacing.sm))
            .py(px(t.spacing.xs))
            .text_size(px(t.typography.size_sm))
            .text_color(hsla(t.palette.text))
            .child(self.text.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_opposite_is_an_involution_on_the_diagonal() {
        for c in Corner::ALL {
            assert_ne!(c, c.opposite());
            assert_eq!(c, c.opposite().opposite());
        }
        assert_eq!(Corner::TopRight.opposite(), Corner::BottomLeft);
        let names: std::collections::HashSet<_> = Corner::ALL.iter().map(|c| c.name()).collect();
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn border_style_sets_width_and_softens_colour() {
        let t = themes::cyberpunk::yellow();
        let base = panel_style(&t, false);
        let s = BorderStyle::new(0.5, 0.5).apply(base);
        assert_eq!(s.stroke_width, 0.5);
        assert!((s.stroke.unwrap().a - base.stroke.unwrap().a * 0.5).abs() < 1e-5);
        assert!(BorderStyle::NONE.apply(base).stroke.is_none());
        let c = BorderStyle::new(99.0, 5.0);
        assert_eq!((c.width, c.opacity), (8.0, 1.0));
        let a = BorderStyle::THIN
            .accent(Some(Corner::TopLeft))
            .accent_width(99.0);
        assert_eq!((a.accent, a.accent_width), (Some(Corner::TopLeft), 8.0));
    }

    use crate::testing::light_rounded;
    use dodai::themes;

    #[test]
    fn panel_focus_changes_border_and_glow() {
        let t = themes::cyberpunk::yellow();
        let a = panel_style(&t, false);
        let b = panel_style(&t, true);
        assert_ne!(a.stroke, b.stroke);
        assert!(a.glow.is_none() && b.glow.is_some());
        assert!(b.shadow.is_some(), "hard_shadow token drives shadow");
    }

    #[test]
    fn panel_adapts_to_light_theme() {
        let t = light_rounded();
        let s = panel_style(&t, true);
        assert_eq!(s.fill, Some(hsla(t.palette.surface)));
        assert!(s.glow.is_none());
    }

    #[test]
    fn input_uses_input_tokens() {
        let t = light_rounded();
        assert_eq!(
            input_style(&t, true).stroke,
            Some(hsla(t.input(true).border))
        );
    }
}
