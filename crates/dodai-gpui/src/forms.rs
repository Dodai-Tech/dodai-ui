//! Form building blocks: [`FormField`] (label, help, error), [`OtpInput`],
//! [`ToggleGroup`] and [`CheckboxGroup`] (checkboxes or switches with select-all).

use crate::button::button_style;
use crate::controls::{Checkbox, Toggle};
use crate::interact::{track, use_interact};
use crate::paint::{shaped, shaped_id};
use crate::panel::input_style;
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::ButtonVariant;
use gpui::{
    div, px, AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ───────────────────────────── form field ─────────────────────────────

/// Validation state shown under a [`FormField`].
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum FieldState {
    #[default]
    Default,
    Error(SharedString),
    Warning(SharedString),
    Success(SharedString),
}

/// Label + control + help or validation message.
///
/// ```ignore
/// FormField::new("Email").required(true).help("We never share it").child(input)
/// ```
#[derive(IntoElement)]
pub struct FormField {
    label: SharedString,
    required: bool,
    help: Option<SharedString>,
    state: FieldState,
    children: Vec<AnyElement>,
}

impl FormField {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            required: false,
            help: None,
            state: FieldState::Default,
            children: Vec::new(),
        }
    }
    /// Append a red `*` to the label.
    pub fn required(mut self, on: bool) -> Self {
        self.required = on;
        self
    }
    /// Muted hint under the control (hidden while a validation message shows).
    pub fn help(mut self, h: impl Into<SharedString>) -> Self {
        self.help = Some(h.into());
        self
    }
    pub fn error(mut self, m: impl Into<SharedString>) -> Self {
        self.state = FieldState::Error(m.into());
        self
    }
    pub fn warning(mut self, m: impl Into<SharedString>) -> Self {
        self.state = FieldState::Warning(m.into());
        self
    }
    pub fn success(mut self, m: impl Into<SharedString>) -> Self {
        self.state = FieldState::Success(m.into());
        self
    }
    pub fn state(mut self, s: FieldState) -> Self {
        self.state = s;
        self
    }
    /// The control(s) being labelled.
    pub fn child(mut self, c: impl IntoElement) -> Self {
        self.children.push(c.into_any_element());
        self
    }
}

impl RenderOnce for FormField {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let p = &t.palette;
        let (msg, color, glyph) = match &self.state {
            FieldState::Error(m) => (Some(m.clone()), p.danger, "✕"),
            FieldState::Warning(m) => (Some(m.clone()), p.warning, "▲"),
            FieldState::Success(m) => (Some(m.clone()), p.success, "✓"),
            FieldState::Default => (self.help.clone(), p.text_muted, ""),
        };
        let mut label = div()
            .flex()
            .gap(px(2.0))
            .text_size(px(t.typography.size_sm))
            .text_color(hsla(if matches!(self.state, FieldState::Error(_)) {
                p.danger
            } else {
                p.text
            }))
            .child(format_label(&t.typography, &self.label));
        if self.required {
            label = label.child(div().text_color(hsla(p.danger)).child("*"));
        }
        let mut col = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .font_family(first_family(t.typography.body))
            .child(label)
            .children(self.children);
        if let Some(m) = msg {
            col = col.child(
                div()
                    .flex()
                    .gap(px(t.spacing.xs))
                    .text_size(px(t.typography.size_xs))
                    .text_color(hsla(color))
                    .child(glyph)
                    .child(m),
            );
        }
        col
    }
}

// ───────────────────────────── one-time code ─────────────────────────────

/// Append the accepted characters of `text` to `buf` (at most `len` in total).
/// `numeric` keeps digits only. Returns whether `buf` changed.
pub fn otp_push(buf: &mut String, len: usize, text: &str, numeric: bool) -> bool {
    let before = buf.len();
    for ch in text.chars() {
        if buf.chars().count() >= len {
            break;
        }
        if ch.is_ascii_alphanumeric() && (!numeric || ch.is_ascii_digit()) {
            buf.push(ch);
        }
    }
    buf.len() != before
}

struct OtpState {
    value: String,
    focus: FocusHandle,
}

type TextFn = Rc<dyn Fn(&str, &mut Window, &mut App)>;

/// Row of single-character cells for verification codes. Type or paste; Backspace deletes.
#[derive(IntoElement)]
pub struct OtpInput {
    id: ElementId,
    len: usize,
    numeric: bool,
    masked: bool,
    on_change: Option<TextFn>,
    on_complete: Option<TextFn>,
}

impl OtpInput {
    pub fn new(id: impl Into<ElementId>, len: usize) -> Self {
        Self {
            id: id.into(),
            len: len.clamp(1, 12),
            numeric: true,
            masked: false,
            on_change: None,
            on_complete: None,
        }
    }
    /// Accept letters as well as digits.
    pub fn alphanumeric(mut self) -> Self {
        self.numeric = false;
        self
    }
    /// Show `•` instead of the character.
    pub fn masked(mut self, on: bool) -> Self {
        self.masked = on;
        self
    }
    crate::callback!(on_change, &str);
    crate::callback!(
        /// Called once all cells are filled.
        on_complete, &str
    );
}

impl RenderOnce for OtpInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, cx| OtpState {
            value: String::new(),
            focus: cx.focus_handle(),
        });
        let (value, focus) = {
            let s = st.read(cx);
            (s.value.clone(), s.focus.clone())
        };
        let focused = focus.is_focused(window);
        let n = value.chars().count();
        let (len, numeric) = (self.len, self.numeric);
        let (on_change, on_complete) = (self.on_change.clone(), self.on_complete.clone());
        let apply = {
            let st = st.clone();
            move |edit: &dyn Fn(&mut String) -> bool, w: &mut Window, cx: &mut App| {
                let mut v = st.read(cx).value.clone();
                if !edit(&mut v) {
                    return;
                }
                st.update(cx, |s, cx| {
                    s.value = v.clone();
                    cx.notify();
                });
                if let Some(f) = &on_change {
                    f(&v, w, cx);
                }
                if v.chars().count() == len {
                    if let Some(f) = &on_complete {
                        f(&v, w, cx);
                    }
                }
            }
        };
        let cell_w = t.shape.control_height * 1.05;
        let mut row = div()
            .id(self.id.clone())
            .flex()
            .gap(px(t.spacing.sm))
            .track_focus(&focus)
            .on_click({
                let focus = focus.clone();
                move |_, w, _| w.focus(&focus)
            })
            .on_key_down(move |e, w, cx| {
                let k = &e.keystroke;
                if k.modifiers.platform || k.modifiers.control {
                    if k.key == "v" {
                        if let Some(text) = cx.read_from_clipboard().and_then(|c| c.text()) {
                            apply(&|b: &mut String| otp_push(b, len, &text, numeric), w, cx);
                        }
                    }
                    return;
                }
                if k.key == "backspace" {
                    apply(&|b: &mut String| b.pop().is_some(), w, cx);
                } else if let Some(ch) = k.key_char.as_deref() {
                    apply(&|b: &mut String| otp_push(b, len, ch, numeric), w, cx);
                }
            });
        let chars: Vec<char> = value.chars().collect();
        for i in 0..self.len {
            let active = focused && i == n.min(self.len - 1);
            let mut cell = shaped(
                &t.role_shape(dodai_core::ShapeRole::Input),
                input_style(&t, active),
            )
            .w(px(cell_w))
            .h(px(t.shape.control_height * 1.3))
            .flex()
            .items_center()
            .justify_center()
            .font_family(first_family(t.typography.display))
            .text_size(px(t.typography.size_xl))
            .text_color(hsla(t.palette.text));
            if let Some(c) = chars.get(i) {
                cell = cell.child(if self.masked {
                    "•".to_string()
                } else {
                    c.to_uppercase().to_string()
                });
            } else if active {
                cell = cell.child(
                    div()
                        .w(px(2.0))
                        .h(px(t.typography.size_xl))
                        .bg(hsla(t.palette.primary)),
                );
            }
            row = row.child(cell);
        }
        row
    }
}

// ───────────────────────────── toggle group ─────────────────────────────

/// Select or deselect `i` in `sel`. Single mode replaces; `allow_empty` lets the
/// last selected item be cleared. Returns whether `sel` changed.
pub fn toggle_select(sel: &mut Vec<usize>, i: usize, multiple: bool, allow_empty: bool) -> bool {
    let on = sel.contains(&i);
    if multiple {
        if on {
            if sel.len() == 1 && !allow_empty {
                return false;
            }
            sel.retain(|&x| x != i);
        } else {
            sel.push(i);
            sel.sort_unstable();
        }
        true
    } else if on {
        if allow_empty {
            sel.clear();
            true
        } else {
            false
        }
    } else {
        *sel = vec![i];
        true
    }
}

type SetFn = Rc<dyn Fn(&[usize], &mut Window, &mut App)>;
type BoolFn = Box<dyn Fn(bool, &mut Window, &mut App)>;

/// Joined row of toggle buttons; single or multiple selection. Keeps its own state.
#[derive(IntoElement)]
pub struct ToggleGroup {
    id: ElementId,
    options: Vec<SharedString>,
    initial: Vec<usize>,
    multiple: bool,
    allow_empty: bool,
    on_change: Option<SetFn>,
}

impl ToggleGroup {
    pub fn new<S: Into<SharedString>>(
        id: impl Into<ElementId>,
        options: impl IntoIterator<Item = S>,
    ) -> Self {
        Self {
            id: id.into(),
            options: options.into_iter().map(Into::into).collect(),
            initial: Vec::new(),
            multiple: false,
            allow_empty: true,
            on_change: None,
        }
    }
    /// Allow several selected at once.
    pub fn multiple(mut self, on: bool) -> Self {
        self.multiple = on;
        self
    }
    /// Initially selected indices.
    pub fn selected(mut self, s: impl IntoIterator<Item = usize>) -> Self {
        self.initial = s.into_iter().collect();
        self
    }
    /// Whether the last selected item can be switched off (default yes).
    pub fn allow_empty(mut self, on: bool) -> Self {
        self.allow_empty = on;
        self
    }
    crate::callback!(on_change, &[usize]);
}

impl RenderOnce for ToggleGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let init = self.initial.clone();
        let st = window.use_keyed_state(self.id.clone(), cx, move |_, _| init);
        let sel = st.read(cx).clone();
        let mut row = div()
            .flex()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
        for (i, label) in self.options.iter().enumerate() {
            let on = sel.contains(&i);
            let id: ElementId = (self.id.clone(), i.to_string()).into();
            let hs = use_interact(&id, window, cx);
            let state = hs.read(cx).control_state(false);
            let variant = if on {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            };
            let (style, fg) = button_style(&t, variant, state);
            let (st2, f, multiple, allow_empty) = (
                st.clone(),
                self.on_change.clone(),
                self.multiple,
                self.allow_empty,
            );
            let mut b = track(shaped_id(id, &t.shape, style), &hs)
                .h(px(t.shape.control_height))
                .px(px(t.spacing.md))
                .flex()
                .items_center()
                .text_color(fg)
                .cursor_pointer()
                .on_click(move |_, w, cx| {
                    let mut v = st2.read(cx).clone();
                    if toggle_select(&mut v, i, multiple, allow_empty) {
                        st2.update(cx, |s, cx| {
                            *s = v.clone();
                            cx.notify();
                        });
                        if let Some(f) = &f {
                            f(&v, w, cx);
                        }
                    }
                })
                .child(format_label(&t.typography, label));
            if i > 0 {
                b = b.ml(px(-t.shape.border_width.max(1.0)));
            }
            row = row.child(b);
        }
        row
    }
}

// ───────────────────────────── checkbox group ─────────────────────────────

/// `(all checked, any checked)` of a set of flags.
pub fn select_all_state(flags: &[bool]) -> (bool, bool) {
    (
        !flags.is_empty() && flags.iter().all(|&f| f),
        flags.iter().any(|&f| f),
    )
}

/// Checkboxes (or switches) with an optional select-all row. Keeps its own state.
#[derive(IntoElement)]
pub struct CheckboxGroup {
    id: ElementId,
    options: Vec<SharedString>,
    initial: Vec<usize>,
    select_all: Option<SharedString>,
    switches: bool,
    on_change: Option<SetFn>,
}

impl CheckboxGroup {
    pub fn new<S: Into<SharedString>>(
        id: impl Into<ElementId>,
        options: impl IntoIterator<Item = S>,
    ) -> Self {
        Self {
            id: id.into(),
            options: options.into_iter().map(Into::into).collect(),
            initial: Vec::new(),
            select_all: None,
            switches: false,
            on_change: None,
        }
    }
    /// Initially checked indices.
    pub fn checked(mut self, c: impl IntoIterator<Item = usize>) -> Self {
        self.initial = c.into_iter().collect();
        self
    }
    /// Add a master row with this label.
    pub fn select_all(mut self, label: impl Into<SharedString>) -> Self {
        self.select_all = Some(label.into());
        self
    }
    /// Use switches instead of checkboxes.
    pub fn switches(mut self, on: bool) -> Self {
        self.switches = on;
        self
    }
    crate::callback!(on_change, &[usize]);
}

impl RenderOnce for CheckboxGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let n = self.options.len();
        let init = self.initial.clone();
        let st = window.use_keyed_state(self.id.clone(), cx, move |_, _| init);
        let sel = st.read(cx).clone();
        let flags: Vec<bool> = (0..n).map(|i| sel.contains(&i)).collect();
        let (all, any) = select_all_state(&flags);
        let set = {
            let st = st.clone();
            let f = self.on_change.clone();
            move |v: Vec<usize>, w: &mut Window, cx: &mut App| {
                st.update(cx, |s, cx| {
                    *s = v.clone();
                    cx.notify();
                });
                if let Some(f) = &f {
                    f(&v, w, cx);
                }
            }
        };
        let control = |id: ElementId, on: bool, mixed: bool, f: BoolFn| {
            if self.switches {
                Toggle::new(id, on).on_change(f).into_any_element()
            } else {
                Checkbox::new(id, on)
                    .indeterminate(mixed)
                    .on_change(f)
                    .into_any_element()
            }
        };
        let row = |ctl: AnyElement, text: SharedString, muted: Option<String>| {
            div()
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .child(ctl)
                .child(div().text_color(hsla(t.palette.text)).child(text))
                .children(muted.map(|m| {
                    div()
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child(m)
                }))
        };
        let mut col = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
        if let Some(label) = self.select_all.clone() {
            let set = set.clone();
            let ctl = control(
                (self.id.clone(), "all").into(),
                all,
                any && !all,
                Box::new(move |on, w, cx| set(if on { (0..n).collect() } else { vec![] }, w, cx)),
            );
            let hint = (any && !all).then(|| format!("{} of {}", sel.len(), n));
            col = col.child(row(ctl, label, hint));
        }
        let mut items = div().flex().flex_col().gap(px(t.spacing.sm));
        if self.select_all.is_some() {
            items = items.pl(px(t.spacing.lg));
        }
        for (i, label) in self.options.iter().enumerate() {
            let set = set.clone();
            let cur = sel.clone();
            let ctl = control(
                (self.id.clone(), format!("o{i}")).into(),
                flags[i],
                false,
                Box::new(move |on, w, cx| {
                    let mut v = cur.clone();
                    v.retain(|&x| x != i);
                    if on {
                        v.push(i);
                        v.sort_unstable();
                    }
                    set(v, w, cx)
                }),
            );
            items = items.child(row(ctl, label.clone(), None));
        }
        col.child(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otp_accepts_only_allowed_chars_up_to_len() {
        let mut b = String::new();
        assert!(otp_push(&mut b, 4, "12a3 4", true));
        assert_eq!(b, "1234");
        assert!(!otp_push(&mut b, 4, "9", true));
        let mut b = String::new();
        otp_push(&mut b, 6, "a1-b2", false);
        assert_eq!(b, "a1b2");
        assert!(!otp_push(&mut String::new(), 3, "xyz", true));
    }

    #[test]
    fn toggle_select_single_and_multiple() {
        let mut s = vec![];
        assert!(toggle_select(&mut s, 1, false, true));
        assert!(toggle_select(&mut s, 2, false, true));
        assert_eq!(s, vec![2]);
        assert!(toggle_select(&mut s, 2, false, true)); // clears
        assert!(s.is_empty());
        assert!(toggle_select(&mut s, 2, false, false));
        assert!(!toggle_select(&mut s, 2, false, false)); // keeps one
        let mut m = vec![];
        toggle_select(&mut m, 3, true, true);
        toggle_select(&mut m, 1, true, true);
        assert_eq!(m, vec![1, 3]);
        toggle_select(&mut m, 1, true, true);
        assert!(!toggle_select(&mut m, 3, true, false));
        assert_eq!(m, vec![3]);
    }

    #[test]
    fn select_all_flags() {
        assert_eq!(select_all_state(&[]), (false, false));
        assert_eq!(select_all_state(&[true, false]), (false, true));
        assert_eq!(select_all_state(&[true, true]), (true, true));
    }
}
