//! [`CommandPalette`] (modal command launcher) and [`Combobox`] (input with a
//! filtered dropdown). Both take a [`TextInput`] entity for the query.

use crate::interact::{track, use_interact};
use crate::list::list_style;
use crate::overlay::{dialog_style, scrim_color};
use crate::paint::{shaped, shaped_id};
use crate::text::first_family;
use crate::text_input::TextInput;
use crate::{hsla, theme_of};
use gpui::{
    deferred, div, px, App, ElementId, Entity, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// Case-insensitive subsequence match. Returns indices of matching entries,
/// best first (prefix matches, then earlier/tighter matches).
pub fn fuzzy_filter<S: AsRef<str>>(query: &str, items: &[S]) -> Vec<usize> {
    let q: Vec<char> = query
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    if q.is_empty() {
        return (0..items.len()).collect();
    }
    let mut scored: Vec<(usize, usize)> = items
        .iter()
        .enumerate()
        .filter_map(|(i, s)| fuzzy_score(&q, &s.as_ref().to_lowercase()).map(|sc| (sc, i)))
        .map(|(sc, i)| (i, sc))
        .collect();
    scored.sort_by_key(|&(i, sc)| (sc, i));
    scored.into_iter().map(|(i, _)| i).collect()
}

/// Lower is better; `None` when `q` is not a subsequence of `s`.
fn fuzzy_score(q: &[char], s: &str) -> Option<usize> {
    let mut pos = 0usize;
    let mut first = None;
    let mut last = 0usize;
    let chars: Vec<char> = s.chars().collect();
    for &c in q {
        let found = chars[pos..].iter().position(|&x| x == c)? + pos;
        first.get_or_insert(found);
        last = found;
        pos = found + 1;
    }
    let first = first.unwrap_or(0);
    let spread = last - first;
    Some(first * 4 + spread)
}

/// One command in a [`CommandPalette`].
#[derive(Clone)]
pub struct Command {
    /// Text shown in the list.
    pub label: SharedString,
    /// Right-aligned hint such as a shortcut.
    pub hint: Option<SharedString>,
}

impl Command {
    /// Create a new [`Command`].
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            hint: None,
        }
    }
    /// Secondary hint text.
    pub fn hint(mut self, h: impl Into<SharedString>) -> Self {
        self.hint = Some(h.into());
        self
    }
}

type IndexFn = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type DismissFn = Rc<dyn Fn(&mut Window, &mut App)>;

/// Modal launcher: query input on top, filtered commands below. `on_run`
/// receives the index into the original `commands`.
#[derive(IntoElement)]
pub struct CommandPalette {
    id: ElementId,
    input: Entity<TextInput>,
    commands: Vec<Command>,
    max_rows: usize,
    on_run: Option<IndexFn>,
    on_dismiss: Option<DismissFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(CommandPalette);

impl CommandPalette {
    /// Create a new [`CommandPalette`].
    pub fn new(id: impl Into<ElementId>, input: Entity<TextInput>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            input,
            commands: Vec::new(),
            max_rows: 8,
            on_run: None,
            on_dismiss: None,
        }
    }
    /// The commands to offer.
    pub fn commands(mut self, c: impl IntoIterator<Item = Command>) -> Self {
        self.commands = c.into_iter().collect();
        self
    }
    /// Most rows shown before scrolling.
    pub fn max_rows(mut self, n: usize) -> Self {
        self.max_rows = n.max(1);
        self
    }
    crate::callback!(
        /// Called when a command is run, with the index.
        on_run, usize
    );
    crate::callback!(
        /// Called when the surface is dismissed.
        on_dismiss
    );
}

impl RenderOnce for CommandPalette {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let query = self.input.read(cx).text().to_string();
        let labels: Vec<&str> = self.commands.iter().map(|c| c.label.as_ref()).collect();
        let hits = fuzzy_filter(&query, &labels);
        // Enter runs the highlighted (first) match.
        if let (Some(&first), Some(f)) = (hits.first(), self.on_run.clone()) {
            self.input
                .update(cx, |i, _| i.set_on_submit(move |_, w, cx| f(first, w, cx)));
        }
        let mut list = div().flex().flex_col();
        for (n, &ci) in hits.iter().take(self.max_rows).enumerate() {
            let c = &self.commands[ci];
            let id: ElementId = (self.id.clone(), format!("c{ci}")).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered;
            let (style, _, marker) = list_style(&t, n == 0, hovered);
            let mut row = track(shaped_id(id, &t.shape, style), &st)
                .relative()
                .flex()
                .items_center()
                .justify_between()
                .h(px(t.shape.control_height * 0.9))
                .px(px(t.spacing.md))
                .text_color(hsla(t.palette.text))
                .cursor_pointer()
                .child(c.label.clone())
                .children(c.hint.clone().map(|h| {
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text_muted))
                        .child(h)
                }));
            if let Some(m) = marker {
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
            if let Some(f) = self.on_run.clone() {
                row = row.on_click(move |_, w, cx| f(ci, w, cx));
            }
            list = list.child(row);
        }
        if hits.is_empty() {
            list = list.child(
                div()
                    .p(px(t.spacing.md))
                    .text_color(hsla(t.palette.text_muted))
                    .child("No matching commands"),
            );
        }
        let card = shaped(
            &t.role_shape(dodai_core::ShapeRole::Dialog),
            dialog_style(&t),
        )
        .w(px(520.0))
        .flex()
        .flex_col()
        .gap(px(t.spacing.sm))
        .p(px(t.spacing.md))
        .font_family(first_family(t.typography.body))
        .text_size(px(t.typography.size_md))
        .child(self.input.clone())
        .child(list);
        let dismiss = self.on_dismiss;
        deferred(
            div()
                .id(self.id)
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .occlude()
                .bg(scrim_color(&t))
                .flex()
                .justify_center()
                .pt(px(80.0))
                .on_mouse_down(MouseButton::Left, move |_, w, cx| {
                    if let Some(f) = &dismiss {
                        f(w, cx);
                    }
                })
                .child(
                    div()
                        .id("palette-card")
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(card),
                ),
        )
        .with_priority(100)
    }
}

/// Text input with a filtered option list shown while `open`.
#[derive(IntoElement)]
pub struct Combobox {
    id: ElementId,
    input: Entity<TextInput>,
    options: Vec<SharedString>,
    open: bool,
    max_rows: usize,
    on_select: Option<IndexFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Combobox);

impl Combobox {
    /// Create a new [`Combobox`].
    pub fn new(id: impl Into<ElementId>, input: Entity<TextInput>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            input,
            options: Vec::new(),
            open: false,
            max_rows: 6,
            on_select: None,
        }
    }
    /// The options to offer.
    pub fn options<S: Into<SharedString>>(mut self, o: impl IntoIterator<Item = S>) -> Self {
        self.options = o.into_iter().map(Into::into).collect();
        self
    }
    /// Whether the popup is open (controlled).
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    crate::callback!(
        /// Called with the index into the original `options`.
        on_select, usize
    );
}

impl Combobox {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let mut root = div().relative().w_full().child(self.input.clone());
        if self.open {
            let query = self.input.read(cx).text().to_string();
            let hits = fuzzy_filter(&query, &self.options);
            let p = t.panel(false);
            let mut menu = shaped(
                &t.shape,
                crate::paint::ShapeStyle::from_tokens(
                    &t,
                    Some(p.background),
                    Some(p.border),
                    false,
                    true,
                ),
            )
            .absolute()
            .top(px(t.shape.control_height + 4.0))
            .left_0()
            .w_full()
            .flex()
            .flex_col()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
            for &oi in hits.iter().take(self.max_rows) {
                let id: ElementId = (self.id.clone(), format!("o{oi}")).into();
                let st = use_interact(&id, window, cx);
                let hovered = st.read(cx).hovered;
                let (style, _, _) = list_style(&t, false, hovered);
                let mut row = track(shaped_id(id, &t.shape, style), &st)
                    .h(px(t.shape.control_height * 0.9))
                    .px(px(t.spacing.md))
                    .flex()
                    .items_center()
                    .text_color(hsla(t.palette.text))
                    .cursor_pointer()
                    .child(self.options[oi].clone());
                if let Some(f) = self.on_select.clone() {
                    row = row.on_click(move |_, w, cx| f(oi, w, cx));
                }
                menu = menu.child(row);
            }
            root = root.child(deferred(menu).with_priority(50));
        }
        root
    }
}

crate::entry_render!(Combobox);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_query_keeps_order() {
        assert_eq!(fuzzy_filter("", &["b", "a", "c"]), vec![0, 1, 2]);
    }

    #[test]
    fn subsequence_and_ranking() {
        let items = [
            "Open File",
            "Close Window",
            "Toggle Fullscreen",
            "Format Document",
        ];
        assert_eq!(fuzzy_filter("fo", &items)[0], 3); // prefix "Fo…" beats later match
        assert_eq!(fuzzy_filter("xyz", &items), Vec::<usize>::new());
        let hits = fuzzy_filter("ow", &items);
        assert!(hits.contains(&1)); // "cl(o)se (w)indow"
        assert!(!fuzzy_filter("OPEN", &items).is_empty());
    }
}
