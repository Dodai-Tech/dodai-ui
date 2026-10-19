//! [`Select`] (dropdown with keyboard navigation) and [`ThemeSelector`]
//! (a `Select` filled from the theme registry).

use crate::interact::{track, use_interact, IndexHandler};
use crate::list::list_style;
use crate::logic::step_index;
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::panel::input_style;
use crate::text::first_family;
use crate::{hsla, select_theme, theme_of, ActiveDodai};
use dodai_core::{Color, Theme};
use gpui::{
    canvas, deferred, div, px, App, Bounds, ElementId, FocusHandle, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Pixels, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};

/// One entry of a [`Select`].
#[derive(Clone)]
pub struct SelectOption {
    pub label: SharedString,
    /// Small colour chips drawn after the label (used by [`ThemeSelector`]).
    pub swatches: Vec<Color>,
}

impl SelectOption {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            swatches: Vec::new(),
        }
    }
    pub fn swatches(mut self, s: impl IntoIterator<Item = Color>) -> Self {
        self.swatches = s.into_iter().collect();
        self
    }
}

impl<S: Into<SharedString>> From<S> for SelectOption {
    fn from(s: S) -> Self {
        Self::new(s)
    }
}

/// Paint style of a floating popup (menus, dropdown lists). Pure.
pub fn popup_style(theme: &Theme) -> ShapeStyle {
    ShapeStyle::from_tokens(
        theme,
        Some(theme.palette.surface_raised),
        Some(theme.palette.border_strong),
        true,
        true,
    )
}

/// Row of small colour chips.
pub fn swatch_row(t: &Theme, colors: &[Color]) -> gpui::Div {
    let mut row = div().flex().gap(px(2.0)).flex_none();
    for c in colors {
        row = row.child(
            shaped(
                &crate::paint::scaled_shape(&t.shape, 12.0),
                ShapeStyle::from_tokens(t, Some(*c), Some(t.palette.border_strong), false, false),
            )
            .w(px(14.0))
            .h(px(14.0)),
        );
    }
    row
}

struct SelectState {
    open: bool,
    highlight: usize,
    focus: FocusHandle,
    trigger: Bounds<Pixels>,
}

/// Dropdown list. Controlled: pass `selected` and update it in `on_change`.
/// Keyboard (trigger focused): ↑/↓ move, Enter/Space open or choose, Esc close.
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    options: Vec<SelectOption>,
    selected: Option<usize>,
    placeholder: SharedString,
    disabled: bool,
    on_change: Option<IndexHandler>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Select);

impl Select {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            options: Vec::new(),
            selected: None,
            placeholder: "Select…".into(),
            disabled: false,
            on_change: None,
        }
    }
    pub fn options<O: Into<SelectOption>>(mut self, o: impl IntoIterator<Item = O>) -> Self {
        self.options = o.into_iter().map(Into::into).collect();
        self
    }
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = p.into();
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    crate::callback!(
        /// Called with the chosen option index.
        on_change, usize
    );
}

impl Select {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let sel0 = self.selected;
        let st = window.use_keyed_state(self.id.clone(), cx, |_, cx| SelectState {
            open: false,
            highlight: sel0.unwrap_or(0),
            focus: cx.focus_handle(),
            trigger: Bounds::default(),
        });
        let (open, highlight, focus, _tb) = {
            let s = st.read(cx);
            (s.open, s.highlight, s.focus.clone(), s.trigger)
        };
        let focused = focus.is_focused(window) && !self.disabled;
        let tid: ElementId = (self.id.clone(), "trigger").into();
        let hov = use_interact(&tid, window, cx);
        let hovered = hov.read(cx).hovered && !self.disabled;
        let n = self.options.len();

        let mut style = input_style(&t, focused || open);
        if self.disabled {
            style.fill = Some(hsla(t.palette.surface_raised));
            style.stroke = Some(hsla(t.palette.border));
        } else if hovered && !(focused || open) {
            style.stroke = Some(hsla(t.palette.primary));
        }
        let current = self.selected.and_then(|i| self.options.get(i)).cloned();
        let text_color = if self.disabled {
            hsla(t.palette.text_disabled)
        } else if current.is_some() {
            hsla(t.palette.text)
        } else {
            hsla(t.palette.text_disabled)
        };

        let on_change = self.on_change.clone();
        let commit = {
            let st = st.clone();
            move |i: usize, w: &mut Window, cx: &mut App| {
                st.update(cx, |s, cx| {
                    s.open = false;
                    s.highlight = i;
                    cx.notify();
                });
                if let Some(f) = &on_change {
                    f(i, w, cx);
                }
            }
        };

        let mut trigger = track(shaped_id(tid, &t.shape, style), &hov)
            .h(px(t.shape.control_height))
            .w_full()
            .px(px(t.spacing.sm))
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(text_color)
            .child(
                canvas(
                    {
                        let st = st.clone();
                        move |b, _, cx| st.update(cx, |s, _| s.trigger = b)
                    },
                    |_, _, _, _| (),
                )
                .absolute()
                .size_full(),
            )
            .child(
                div().flex_1().overflow_hidden().whitespace_nowrap().child(
                    current
                        .as_ref()
                        .map_or(self.placeholder.clone(), |o| o.label.clone()),
                ),
            );
        if let Some(o) = &current {
            if !o.swatches.is_empty() {
                trigger = trigger.child(swatch_row(&t, &o.swatches));
            }
        }
        trigger = trigger.child(
            div()
                .flex_none()
                .text_color(hsla(if open {
                    t.palette.primary
                } else {
                    t.palette.text_muted
                }))
                .child(if open { "▴" } else { "▾" }),
        );

        if !self.disabled {
            trigger = trigger
                .cursor_pointer()
                .track_focus(&focus)
                .on_click({
                    let st = st.clone();
                    let focus = focus.clone();
                    let sel = self.selected;
                    move |_, w, cx| {
                        w.focus(&focus);
                        st.update(cx, |s, cx| {
                            s.open = !s.open;
                            if s.open {
                                s.highlight = sel.unwrap_or(s.highlight);
                            }
                            cx.notify();
                        });
                    }
                })
                .on_key_down({
                    let st = st.clone();
                    let commit = commit.clone();
                    let sel = self.selected;
                    move |e, w, cx| {
                        let key = e.keystroke.key.as_str();
                        let (open, hi) = {
                            let s = st.read(cx);
                            (s.open, s.highlight)
                        };
                        let mv = |delta: i32, cx: &mut App| {
                            st.update(cx, |s, cx| {
                                if !s.open {
                                    s.open = true;
                                    s.highlight = sel.unwrap_or(0);
                                } else if let Some(i) = step_index(Some(hi), n, delta, true) {
                                    s.highlight = i;
                                }
                                cx.notify();
                            });
                        };
                        match key {
                            "down" => mv(1, cx),
                            "up" => mv(-1, cx),
                            "home" if open => st.update(cx, |s, cx| {
                                s.highlight = 0;
                                cx.notify();
                            }),
                            "end" if open => st.update(cx, |s, cx| {
                                s.highlight = n.saturating_sub(1);
                                cx.notify();
                            }),
                            "enter" | "space" if open && n > 0 => commit(hi.min(n - 1), w, cx),
                            "enter" | "space" => mv(1, cx),
                            "escape" | "tab" if open => st.update(cx, |s, cx| {
                                s.open = false;
                                cx.notify();
                            }),
                            _ => {}
                        }
                    }
                });
        }

        let mut root = div().relative().w_full().child(trigger);
        if open && !self.disabled {
            let mut rows = div()
                .id((self.id.clone(), "rows"))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .max_h(px(280.0))
                .overflow_y_scroll();
            for (i, o) in self.options.iter().enumerate() {
                let is_sel = self.selected == Some(i);
                let (rs, fg, marker) = list_style(&t, is_sel || highlight == i, false);
                let fg = if is_sel || highlight == i {
                    hsla(t.palette.text)
                } else {
                    fg
                };
                let c = commit.clone();
                let hs = st.clone();
                let mut row = shaped(&t.shape, rs)
                    .id((self.id.clone(), format!("opt{i}")))
                    .h(px(t.shape.control_height * 0.8))
                    .px(px(t.spacing.md))
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.sm))
                    .cursor_pointer()
                    .text_color(fg)
                    .on_hover(move |h, _, cx| {
                        if *h {
                            hs.update(cx, |s, cx| {
                                if s.highlight != i {
                                    s.highlight = i;
                                    cx.notify();
                                }
                            });
                        }
                    })
                    .on_click(move |_, w, cx| c(i, w, cx))
                    .child(div().flex_1().whitespace_nowrap().child(o.label.clone()));
                if !o.swatches.is_empty() {
                    row = row.child(swatch_row(&t, &o.swatches));
                }
                if let (true, Some(m)) = (is_sel, marker) {
                    row = row.child(
                        div()
                            .absolute()
                            .left_0()
                            .top_0()
                            .bottom_0()
                            .w(px(t.shape.border_width.max(2.0)))
                            .bg(m),
                    );
                }
                rows = rows.child(row);
            }
            let out = st.clone();
            let popup = shaped(&t.shape, popup_style(&t))
                .absolute()
                .top(px(t.shape.control_height + 4.0))
                .left_0()
                .w_full()
                .min_w(px(180.0))
                .p(px(t.spacing.xs + 2.0))
                .occlude()
                .font_family(first_family(t.typography.body))
                .text_size(px(t.typography.size_md))
                .on_mouse_down_out(move |e, _, cx| {
                    let inside = out.read(cx).trigger.contains(&e.position);
                    if !inside {
                        out.update(cx, |s, cx| {
                            s.open = false;
                            cx.notify();
                        });
                    }
                })
                .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                .child(rows);
            root = root.child(deferred(popup).with_priority(10));
        }
        root
    }
}

crate::entry_render!(Select);

/// Option list for a theme registry: name + colour swatches per theme.
pub fn theme_options(themes: &[Theme]) -> Vec<SelectOption> {
    themes
        .iter()
        .map(|t| {
            let p = &t.palette;
            SelectOption::new(t.name()).swatches([p.background, p.primary, p.secondary, p.accent])
        })
        .collect()
}

/// Display title of a family id (`"cyberpunk"` → `"Cyberpunk"`).
pub fn family_title(id: &str) -> String {
    let mut c = id.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

/// Dropdown of the variants (sub-themes) of the active design system;
/// choosing one applies it live. New variants appear automatically.
#[derive(IntoElement)]
pub struct ThemeSelector {
    id: ElementId,
}

impl ThemeSelector {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
}

impl RenderOnce for ThemeSelector {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let lt = cx.dodai();
        let sw = lt.switcher();
        let idx = sw.family_indices(sw.current_family());
        let list: Vec<Theme> = idx.iter().map(|&i| sw.themes()[i].clone()).collect();
        let pos = idx.iter().position(|&i| i == sw.index());
        Select::new(self.id)
            .options(theme_options(&list))
            .selected(pos)
            .on_change(move |i, _, cx| select_theme(cx, idx[i]))
    }
}

/// Dropdown of registered design systems (families). Choosing one switches
/// to its first variant.
#[derive(IntoElement)]
pub struct DesignSystemSelector {
    id: ElementId,
}

impl DesignSystemSelector {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
}

impl RenderOnce for DesignSystemSelector {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let lt = cx.dodai();
        let sw = lt.switcher();
        let fams = sw.families();
        let opts: Vec<SelectOption> = fams
            .iter()
            .map(|f| {
                let i = sw.family_indices(f)[0];
                let p = &sw.themes()[i].palette;
                SelectOption::new(family_title(f)).swatches([
                    p.background,
                    p.primary,
                    p.secondary,
                    p.accent,
                ])
            })
            .collect();
        let cur = fams.iter().position(|f| *f == sw.current_family());
        Select::new(self.id)
            .options(opts)
            .selected(cur)
            .on_change(move |i, _, cx| {
                let idx = cx.dodai().switcher().family_indices(fams[i])[0];
                select_theme(cx, idx)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};
    use dodai::themes;

    #[test]
    fn theme_options_cover_registry() {
        let all = themes::all();
        let o = theme_options(&all);
        assert_eq!(o.len(), all.len());
        for (opt, t) in o.iter().zip(&all) {
            assert_eq!(opt.label.as_ref(), t.name());
            assert_eq!(opt.swatches.len(), 4);
        }
        let mut extended = all.clone();
        extended.push(light_rounded());
        extended.push(square());
        assert_eq!(theme_options(&extended).len(), all.len() + 2);
    }

    #[test]
    fn popup_follows_effect_tokens() {
        let l = popup_style(&light_rounded());
        assert!(l.glow.is_none() && l.shadow.is_none());
        let q = popup_style(&square());
        assert!(q.shadow.is_some());
        assert!(popup_style(&themes::cyberpunk::night_city()).glow.is_some());
    }
}
