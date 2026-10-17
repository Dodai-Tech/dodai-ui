//! [`Panel`] (card), [`InputBox`] (text-input container) and [`DodaiTooltip`].

use crate::paint::{shaped, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, px, AnyElement, AnyView, App, AppContext as _, Context, InteractiveElement as _,
    IntoElement, ParentElement, Render, RenderOnce, SharedString, Styled, Window,
};

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
}

impl Panel {
    pub fn new() -> Self {
        Self {
            title: None,
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

impl Panel {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let p = t.panel(self.focused);
        let mut d = shaped(&t.shape, panel_style(&t, self.focused))
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .p(px(t.spacing.lg))
            .text_color(hsla(t.palette.text))
            .text_size(px(t.typography.size_md))
            .font_family(first_family(t.typography.body));
        if let Some(title) = self.title {
            d = d.child(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(p.title))
                    .child(format_label(&t.typography, &title)),
            );
        }
        if let Some(sh) = self.subheader {
            d = d.child(sh);
        }
        match self.scope {
            Some(set) => {
                let key = self.scope_key.wrapping_mul(256);
                d = d.children(self.children.into_iter().enumerate().map(|(i, c)| {
                    div()
                        .id(("pscope", key.wrapping_add(i as u32)))
                        .flex()
                        .flex_col()
                        .child(crate::motion::EntryScope::new(set, c).delay(
                            std::time::Duration::from_millis(set.stagger_ms() as u64 * i as u64),
                        ))
                }));
            }
            None => d = d.children(self.children),
        }
        match self.refine {
            Some(f) => f(d),
            None => d,
        }
    }
}

impl RenderOnce for Panel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Panel", motion)
    }
}

/// Styled container for a text-input entity. gpui 0.2.2 ships no text input;
/// put your own editor view/entity inside (or use `.placeholder`).
#[derive(IntoElement)]
pub struct InputBox {
    focused: bool,
    placeholder: Option<SharedString>,
    children: Vec<AnyElement>,
}

impl InputBox {
    pub fn new() -> Self {
        Self {
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

impl RenderOnce for InputBox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "InputBox", motion)
    }
}

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
    use crate::testing::light_rounded;
    use dodai_core::themes;

    #[test]
    fn panel_focus_changes_border_and_glow() {
        let t = themes::cyberpunk::night_city();
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
