//! [`Select`] (dropdown with keyboard navigation) and [`ThemeSelector`]
//! (a `Select` filled from the theme registry).

use crate::interact::{track, use_interact, IndexHandler};
use crate::list::list_style;
use crate::logic::step_index;
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::panel::input_style;
use crate::text::first_family;
use crate::{hsla, select_theme, theme_of, ActiveDodai};
use dodai_ui_core::{Color, Theme};
use gpui::{
    canvas, deferred, div, px, App, Bounds, ElementId, Entity, FocusHandle, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Pixels, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};

/// Commits the option at an index (closes the list, then notifies the owner).
type Commit = std::rc::Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// One entry of a [`Select`].
#[derive(Clone)]
pub struct SelectOption {
    /// Text shown in the list and, when chosen, in the field.
    pub label: SharedString,
    /// Small colour chips drawn after the label (used by [`ThemeSelector`]).
    pub swatches: Vec<Color>,
}

impl SelectOption {
    /// Create a new [`SelectOption`].
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            swatches: Vec::new(),
        }
    }
    /// Colour swatches shown beside the label.
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
    ShapeStyle::from_surface(theme, theme.popup()).0
}

/// Row of small colour chips.
pub fn swatch_row(t: &Theme, colors: &[Color]) -> gpui::Div {
    let swatch = t.sizes().legend_swatch * 1.2;
    let mut row = div().flex().gap(px(t.spacing.half())).flex_none();
    for c in colors {
        row = row.child(
            shaped(
                &crate::paint::scaled_shape(&t.shape, swatch),
                ShapeStyle::from_tokens(t, Some(*c), Some(t.palette.border_strong), false, false),
            )
            .w(px(swatch))
            .h(px(swatch)),
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

/// Dropdown list. Controlled: pass `selected` and update it in `on_select`.
/// Keyboard (trigger focused): ↑/↓ move, Enter/Space open or choose, Esc close.
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    options: Vec<SelectOption>,
    selected: Option<usize>,
    placeholder: SharedString,
    disabled: bool,
    on_select: Option<IndexHandler>,
    corners: Option<dodai_ui_core::Corners>,
    surface: dodai_ui_core::Surface,
    popup_surface: dodai_ui_core::Surface,
    list_scale: f32,
}

crate::corners_setter!(Select);
crate::surface_setter!(Select);

impl Select {
    /// Create a new [`Select`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            surface: dodai_ui_core::Surface::INHERIT,
            popup_surface: dodai_ui_core::Surface::INHERIT,
            id: id.into(),
            options: Vec::new(),
            selected: None,
            placeholder: "Select…".into(),
            disabled: false,
            on_select: None,
            list_scale: 1.0,
        }
    }
    /// Frost and opacity of the open list. By default it follows the app-wide surface
    /// (a frosted app frosts its popups); `Surface::new().frost(v).opacity(v)` sets the
    /// intensity for this list alone, `Surface::NONE` keeps it solid.
    pub fn popup_surface(mut self, s: dodai_ui_core::Surface) -> Self {
        self.popup_surface = s;
        self
    }
    /// Let the open list grow to `factor` times its usual height before it scrolls, for a
    /// long list whose last entries would otherwise hide below the fold.
    pub fn list_height(mut self, factor: f32) -> Self {
        self.list_scale = factor.max(1.0);
        self
    }
    /// The options to offer.
    pub fn options<O: Into<SelectOption>>(mut self, o: impl IntoIterator<Item = O>) -> Self {
        self.options = o.into_iter().map(Into::into).collect();
        self
    }
    /// The selected item (controlled).
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    /// Placeholder text.
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = p.into();
        self
    }
    /// Disable the control.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    crate::callback!(
        /// Called with the chosen option index.
        on_select,
        usize
    );
}

/// Keyboard on the trigger: arrows move (opening the list first), Enter/Space choose or open,
/// Home/End jump, Escape/Tab close.
fn key_down(
    st: &Entity<SelectState>,
    commit: &Commit,
    sel: Option<usize>,
    n: usize,
    key: &str,
    w: &mut Window,
    cx: &mut App,
) {
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

impl Select {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.selected);
        self.selected = own.get();
        let (state, inner) = (own.setter(), self.on_select.take());
        self.on_select = Some(std::rc::Rc::new(move |i, w, cx| {
            crate::own::choose(&state, cx, Some(i));
            if let Some(f) = &inner {
                f(i, w, cx);
            }
        }));
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

        let mut style = input_style(&t, focused || open).with_own_surface(
            &t,
            dodai_ui_core::SurfaceClass::Input,
            &self.surface,
        );
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

        let on_select = self.on_select.clone();
        let commit: Commit = {
            let st = st.clone();
            std::rc::Rc::new(move |i: usize, w: &mut Window, cx: &mut App| {
                st.update(cx, |s, cx| {
                    s.open = false;
                    s.highlight = i;
                    cx.notify();
                });
                if let Some(f) = &on_select {
                    f(i, w, cx);
                }
            })
        };

        let mut trigger = track(
            shaped_id(tid, &t.role_shape(dodai_ui_core::ShapeRole::Input), style),
            &hov,
            cx,
        )
        .min_h(px(t.shape.control_height))
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
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .child(
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
        trigger = trigger.child(div().flex_none().child(crate::surfaces::glyph_el(
            if open { "▴" } else { "▾" }.into(),
            t.sizes().icon_sm,
            hsla(if open {
                t.palette.primary
            } else {
                t.palette.text_muted
            }),
        )));

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
                    move |e, w, cx| key_down(&st, &commit, sel, n, e.keystroke.key.as_str(), w, cx)
                });
        }

        let mut root = div().relative().w_full().child(trigger);
        if open && !self.disabled {
            let popup = self.popup(&t, &st, highlight, &commit);
            root = root.child(deferred(popup).with_priority(10));
        }
        root
    }
}

impl Select {
    /// The open option list: scrolling rows with a highlight, in a deferred popup.
    fn popup(
        &self,
        t: &Theme,
        st: &Entity<SelectState>,
        highlight: usize,
        commit: &Commit,
    ) -> impl IntoElement {
        let mut rows = div()
            .id((self.id.clone(), "rows"))
            .flex()
            .flex_col()
            .gap(px(t.spacing.half()))
            .max_h(px(t.scale(dodai_ui_core::Scale::ListMax) * self.list_scale))
            .overflow_y_scroll();
        for (i, o) in self.options.iter().enumerate() {
            let is_sel = self.selected == Some(i);
            let (rs, fg, marker) = list_style(t, is_sel || highlight == i, false);
            let fg = if is_sel || highlight == i {
                hsla(t.palette.text)
            } else {
                fg
            };
            let c = commit.clone();
            let hs = st.clone();
            let mut row = shaped(&t.shape, rs)
                .id((self.id.clone(), format!("opt{i}")))
                .min_h(px(t.menu_row_height()))
                .py(px(t.spacing.half()))
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
                row = row.child(swatch_row(t, &o.swatches));
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
        let popup = shaped(
            &t.role_shape(dodai_ui_core::ShapeRole::Menu),
            popup_style(t).with_surface_of(
                t,
                dodai_ui_core::SurfaceClass::Panel,
                &self.popup_surface,
            ),
        )
        .absolute()
        .top(px(t.shape.control_height + 4.0))
        .left_0()
        .w_full()
        .min_w(px(t.scale(dodai_ui_core::Scale::Panel)))
        .p(px(t.spacing.xs + t.spacing.half()))
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
        popup
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
    system: bool,
    popup_surface: dodai_ui_core::Surface,
}

impl ThemeSelector {
    /// Create a new [`ThemeSelector`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            system: false,
            popup_surface: dodai_ui_core::Surface::INHERIT,
        }
    }

    /// Frost and opacity of the open list, see [`Select::popup_surface`].
    pub fn popup_surface(mut self, s: dodai_ui_core::Surface) -> Self {
        self.popup_surface = s;
        self
    }

    /// Offer "System" as the first option: follow the operating system's light or dark
    /// ([`crate::follow_system_appearance`]) instead of a fixed theme.
    pub fn system(mut self, on: bool) -> Self {
        self.system = on;
        self
    }
}

impl RenderOnce for ThemeSelector {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let lt = cx.dodai();
        let sw = lt.switcher();
        let idx = sw.family_indices(sw.current_family());
        let list: Vec<Theme> = idx.iter().map(|&i| sw.themes()[i].clone()).collect();
        let pos = idx.iter().position(|&i| i == sw.index());
        let system = self.system;
        let mut options = theme_options(&list);
        let selected = if system {
            options.insert(0, SelectOption::new("System"));
            if crate::following_system_appearance() {
                Some(0)
            } else {
                pos.map(|p| p + 1)
            }
        } else {
            pos
        };
        Select::new(self.id)
            .popup_surface(self.popup_surface)
            .options(options)
            .selected(selected)
            .on_select(move |i, w, cx| {
                if system && i == 0 {
                    crate::follow_system_appearance(w, cx);
                } else {
                    select_theme(cx, idx[i - usize::from(system)])
                }
            })
    }
}

/// Dropdown of registered design systems (families). Choosing one switches
/// to its first variant.
#[derive(IntoElement)]
pub struct DesignSystemSelector {
    id: ElementId,
    popup_surface: dodai_ui_core::Surface,
}

impl DesignSystemSelector {
    /// Create a new [`DesignSystemSelector`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            popup_surface: dodai_ui_core::Surface::INHERIT,
        }
    }

    /// Frost and opacity of the open list, see [`Select::popup_surface`].
    pub fn popup_surface(mut self, s: dodai_ui_core::Surface) -> Self {
        self.popup_surface = s;
        self
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
            .popup_surface(self.popup_surface)
            .options(opts)
            .selected(cur)
            .on_select(move |i, _, cx| {
                let idx = cx.dodai().switcher().family_indices(fams[i])[0];
                select_theme(cx, idx)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};
    use dodai_ui_core::themes;

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
    fn dodai_dark_popup_is_opaque() {
        let t = themes::all()
            .into_iter()
            .find(|t| t.name() == "Dodai · Dark")
            .unwrap();
        assert!((popup_style(&t).fill.unwrap().a - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn popup_follows_effect_tokens() {
        let l = popup_style(&light_rounded());
        assert!(l.glow.is_none() && l.shadow.is_none());
        let q = popup_style(&square());
        assert!(q.shadow.is_some());
        assert!(popup_style(&themes::cyberpunk::yellow()).glow.is_some());
    }
}
