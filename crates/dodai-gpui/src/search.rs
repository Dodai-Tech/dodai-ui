//! [`SearchView`] (text input + live results), [`RichTooltip`] (title, body,
//! optional action) and [`IconBadge`] (badge pinned to a child's corner).

use crate::command::fuzzy_filter;
use crate::data::Badge;
use crate::interact::{track, use_interact};
use crate::list::list_style;
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::text::first_family;
use crate::text_input::TextInput;
use crate::{hsla, theme_of};
use dodai_core::Tone;
use gpui::{
    deferred, div, px, AnyElement, AnyView, App, AppContext as _, Context, ElementId, Entity,
    InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

type IndexFn = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type VoidFn = Rc<dyn Fn(&mut Window, &mut App)>;

// ───────────────────────────── SearchView ─────────────────────────────

/// Split `text` into `(segment, matched)` runs for highlighting the fuzzy
/// subsequence match of `query` (case-insensitive). Pure.
pub fn highlight_runs(text: &str, query: &str) -> Vec<(String, bool)> {
    let q: Vec<char> = query.to_lowercase().chars().collect();
    let mut runs: Vec<(String, bool)> = Vec::new();
    let mut qi = 0;
    for c in text.chars() {
        let hit = qi < q.len() && c.to_lowercase().next() == Some(q[qi]);
        if hit {
            qi += 1;
        }
        match runs.last_mut() {
            Some((s, m)) if *m == hit => s.push(c),
            _ => runs.push((c.to_string(), hit)),
        }
    }
    if qi < q.len() {
        // Not a full match: no highlighting.
        return vec![(text.to_string(), false)];
    }
    runs
}

/// Where a [`SearchView`] opens (Material 3 search view modes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchMode {
    /// Results drop down from the search bar in one rounded container.
    #[default]
    Docked,
    /// The view fills its (relatively positioned) parent: back arrow, input, results.
    FullScreen,
}

/// Full-text search: input, clear button and a fuzzy-filtered result list
/// with matched characters highlighted. Owns no state: pass the
/// [`TextInput`] entity and an `open` flag.
#[derive(IntoElement)]
pub struct SearchView {
    id: ElementId,
    input: Entity<TextInput>,
    items: Vec<SharedString>,
    open: bool,
    max_rows: usize,
    recent: usize,
    mode: SearchMode,
    on_select: Option<IndexFn>,
    on_clear: Option<VoidFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(SearchView);

impl SearchView {
    /// Create a new [`SearchView`].
    pub fn new(id: impl Into<ElementId>, input: Entity<TextInput>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            input,
            items: Vec::new(),
            open: false,
            max_rows: 6,
            recent: 0,
            mode: SearchMode::Docked,
            on_select: None,
            on_clear: None,
        }
    }
    /// The items.
    pub fn items<S: Into<SharedString>>(mut self, i: impl IntoIterator<Item = S>) -> Self {
        self.items = i.into_iter().map(Into::into).collect();
        self
    }
    /// Whether the popup is open (controlled).
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    /// Docked or full-screen; Material themes only (others always drop down).
    pub fn mode(mut self, m: SearchMode) -> Self {
        self.mode = m;
        self
    }
    /// The first `n` items are recent searches and get the history icon (Material 3).
    pub fn recent(mut self, n: usize) -> Self {
        self.recent = n;
        self
    }
    /// Most rows shown before scrolling.
    pub fn max_rows(mut self, n: usize) -> Self {
        self.max_rows = n.max(1);
        self
    }
    crate::callback!(
        /// Called with the index into the original `items`.
        on_select, usize
    );
    crate::callback!(
        /// Show a ✕ button while the query is non-empty.
        on_clear
    );
}

impl SearchView {
    /// Material 3 search view: one tonal container holding the header (search icon or back
    /// arrow, input, clear), a divider and the result rows.
    fn render_m3(
        self,
        t: dodai_core::Theme,
        query: String,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        use crate::icons::{Icon, IconName};
        let r = t.palette.roles.clone();
        let full = self.mode == SearchMode::FullScreen;
        let head_h = if full { 72.0 } else { 56.0 };
        let fill = if full {
            t.palette.background
        } else {
            r.surface_container_high
        };
        let mut shape = t.role_shape(dodai_core::ShapeRole::SearchBar);
        if full {
            shape.radius = 0.0;
        } else if self.open {
            shape.radius = 28.0;
        }
        let style = ShapeStyle::from_tokens(&t, Some(fill), None, false, false);
        let mut head = div()
            .h(px(head_h))
            .px(px(16.0))
            .flex()
            .items_center()
            .gap(px(16.0))
            .child(
                Icon::new(if full {
                    IconName::ArrowLeft
                } else {
                    IconName::Search
                })
                .size(24.0)
                .color(hsla(r.on_surface)),
            )
            .child(div().flex_1().child(self.input.clone()));
        if let (false, Some(f)) = (query.is_empty(), self.on_clear.clone()) {
            head = head.child(
                div()
                    .id((self.id.clone(), "clear"))
                    .cursor_pointer()
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(Icon::Close.size(24.0).color(hsla(r.on_surface_variant))),
            );
        }
        let mut body = shaped(&shape, style)
            .w_full()
            .flex()
            .flex_col()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .child(head);
        if full {
            body = body.size_full();
        }
        if self.open || full {
            let hits = fuzzy_filter(&query, &self.items);
            body = body.child(div().h(px(1.0)).w_full().bg(hsla(r.outline)));
            if hits.is_empty() {
                body = body.child(
                    div()
                        .h(px(56.0))
                        .px(px(16.0))
                        .flex()
                        .items_center()
                        .text_color(hsla(r.on_surface_variant))
                        .child(SharedString::from("No results")),
                );
            }
            let limit = if full { usize::MAX } else { self.max_rows };
            for &i in hits.iter().take(limit) {
                let id: ElementId = (self.id.clone(), format!("r{i}")).into();
                let st = use_interact(&id, window, cx);
                let hovered = st.read(cx).hovered;
                let mut row = track(gpui::InteractiveElement::id(div(), id), &st)
                    .h(px(t.list_row_height(false)))
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .cursor_pointer()
                    .child(
                        Icon::new(if i < self.recent {
                            IconName::History
                        } else {
                            IconName::Search
                        })
                        .size(24.0)
                        .color(hsla(r.on_surface_variant)),
                    );
                if hovered {
                    row = row.bg(hsla(r.on_surface.with_alpha(0.08)));
                }
                let mut label = div().flex();
                for (seg, hit) in highlight_runs(&self.items[i], &query) {
                    label = label.child(
                        div()
                            .text_color(hsla(if hit { t.palette.primary } else { r.on_surface }))
                            .child(seg),
                    );
                }
                row = row.child(label);
                if let Some(f) = self.on_select.clone() {
                    row = row.on_click(move |_, w, cx| f(i, w, cx));
                }
                body = body.child(row);
            }
        }
        if full {
            return div()
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .child(body)
                .into_any_element();
        }
        // Docked: the container floats over the page; the slot keeps the bar's height.
        div()
            .relative()
            .w_full()
            .h(px(head_h))
            .child(
                deferred(div().absolute().top_0().left_0().w_full().child(body)).with_priority(50),
            )
            .into_any_element()
    }
}

impl RenderOnce for SearchView {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let query = self.input.read(cx).text().to_string();
        let rounded = t.look().tonal;
        // Material draws its own container around the field.
        self.input.update(cx, |i, _| i.bare = rounded);
        if rounded {
            return self.render_m3(t, query, window, cx).into_any_element();
        }
        let mut head = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .child(
                crate::icons::Icon::Search
                    .size(t.typography.size_md * 1.1)
                    .color(hsla(t.palette.text_muted)),
            )
            .child(div().flex_1().child(self.input.clone()));
        if let (false, Some(f)) = (query.is_empty(), self.on_clear.clone()) {
            head = head.child(
                div()
                    .id((self.id.clone(), "clear"))
                    .cursor_pointer()
                    .text_color(hsla(t.palette.text_muted))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        crate::icons::Icon::Close
                            .size(t.typography.size_md)
                            .color(hsla(t.palette.text_muted)),
                    ),
            );
        }
        let mut root = div().relative().w_full().child(head);
        if self.open {
            let hits = fuzzy_filter(&query, &self.items);
            let p = t.panel(false);
            let mut menu = shaped(
                &t.shape,
                ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border), false, true),
            )
            .absolute()
            .top(px(t.shape.control_height + 4.0))
            .left_0()
            .w_full()
            .flex()
            .flex_col()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
            if hits.is_empty() {
                menu = menu.child(
                    div()
                        .px(px(t.spacing.md))
                        .py(px(t.spacing.sm))
                        .text_color(hsla(t.palette.text_muted))
                        .child(SharedString::from("No results")),
                );
            }
            for &i in hits.iter().take(self.max_rows) {
                let id: ElementId = (self.id.clone(), format!("r{i}")).into();
                let st = use_interact(&id, window, cx);
                let hovered = st.read(cx).hovered;
                let (style, _, _) = list_style(&t, false, hovered);
                let mut row = track(shaped_id(id, &t.shape, style), &st)
                    .h(px(t.shape.control_height * 0.9))
                    .px(px(t.spacing.md))
                    .flex()
                    .items_center()
                    .cursor_pointer();
                for (seg, hit) in highlight_runs(&self.items[i], &query) {
                    row = row.child(
                        div()
                            .text_color(hsla(if hit {
                                t.palette.primary
                            } else {
                                t.palette.text
                            }))
                            .child(seg),
                    );
                }
                if let Some(f) = self.on_select.clone() {
                    row = row.on_click(move |_, w, cx| f(i, w, cx));
                }
                menu = menu.child(row);
            }
            root = root.child(deferred(menu).with_priority(50));
        }
        root.into_any_element()
    }
}

// ───────────────────────────── RichTooltip ─────────────────────────────

/// Tooltip with a title, body text and optional action label:
/// `.tooltip(RichTooltip::build("Title", "Body", Some("Learn more")))`.
pub struct RichTooltip {
    title: SharedString,
    body: SharedString,
    action: Option<SharedString>,
}

impl RichTooltip {
    /// Builder for gpui's `.tooltip(...)`.
    pub fn build(
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
        action: Option<&str>,
    ) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
        let (title, body) = (title.into(), body.into());
        let action = action.map(|a| SharedString::from(a.to_string()));
        move |_, cx| {
            let (title, body, action) = (title.clone(), body.clone(), action.clone());
            cx.new(|_| RichTooltip {
                title,
                body,
                action,
            })
            .into()
        }
    }
}

impl RichTooltip {
    /// The tooltip as an ordinary view (static previews, docs).
    pub fn view(
        title: impl Into<SharedString>,
        body: impl Into<SharedString>,
        action: Option<&str>,
        cx: &mut App,
    ) -> AnyView {
        let (title, body) = (title.into(), body.into());
        let action = action.map(|a| SharedString::from(a.to_string()));
        cx.new(|_| RichTooltip {
            title,
            body,
            action,
        })
        .into()
    }
}

impl Render for RichTooltip {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme_of(cx);
        let rounded = t.look().tonal;
        let (bg, _) = t.tooltip(true);
        let r = t.palette.roles.clone();
        // M3 rich tooltip: tonal container, 12 px corners, elevation 2, 16 px / 12 px padding,
        // title in `on-surface-variant`, body and action as labels.
        let (fill, title_c, body_c) = if rounded {
            (bg, r.on_surface_variant, r.on_surface_variant)
        } else {
            (bg, t.palette.text, t.palette.text_muted)
        };
        let mut style = ShapeStyle::from_tokens(
            &t,
            Some(fill),
            if rounded {
                None
            } else {
                Some(t.palette.border_strong)
            },
            false,
            !rounded,
        );
        let shape = if rounded {
            style = style.with_lift(&t, 2);
            t.role_shape(dodai_core::ShapeRole::RichTooltip)
        } else {
            t.shape.clone()
        };
        let (px_, py_) = if rounded {
            (16.0, 12.0)
        } else {
            (t.spacing.md, t.spacing.md)
        };
        let mut el = shaped(&shape, style)
            .max_w(px(if rounded { 320.0 } else { 280.0 }))
            .px(px(px_))
            .py(px(py_))
            .flex()
            .flex_col()
            .gap(px(if rounded { 4.0 } else { t.spacing.xs }))
            .font_family(first_family(t.typography.body))
            .child(
                div()
                    .text_size(px(t.typography.size_md))
                    .text_color(hsla(title_c))
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(body_c))
                    .child(self.body.clone()),
            );
        if let Some(a) = self.action.clone() {
            el = el.child(
                div()
                    .mt(px(if rounded { 8.0 } else { 0.0 }))
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.primary))
                    .child(a),
            );
        }
        el
    }
}

// ────────────────────────────── IconBadge ──────────────────────────────

/// Pins a [`Badge`] to the top-right corner of `child` (icon, avatar, …).
#[derive(IntoElement)]
pub struct IconBadge {
    child: AnyElement,
    badge: Option<Badge>,
}

impl IconBadge {
    /// Create a new [`IconBadge`].
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
            badge: None,
        }
    }
    /// Counter badge; shows `99+` past 99, hidden when `n == 0`.
    pub fn count(mut self, n: u32, tone: Tone) -> Self {
        self.badge = (n > 0).then(|| Badge::new(count_label(n), tone));
        self
    }
    /// Status dot.
    pub fn dot(mut self, tone: Tone) -> Self {
        self.badge = Some(Badge::dot(tone));
        self
    }
}

/// `n` as badge text (`99+` cap). Pure.
pub fn count_label(n: u32) -> String {
    if n > 99 {
        "99+".into()
    } else {
        n.to_string()
    }
}

impl IconBadge {
    fn build(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut el = div().relative().child(self.child);
        if let Some(b) = self.badge {
            el = el.child(div().absolute().top(px(-6.0)).right(px(-6.0)).child(b));
        }
        el
    }
}

crate::entry_render!(IconBadge);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_marks_subsequence() {
        let r = highlight_runs("Open File", "of");
        assert_eq!(r[0], ("O".to_string(), true));
        assert!(r.iter().any(|(s, m)| *m && s == "F"));
        let joined: String = r.iter().map(|(s, _)| s.as_str()).collect();
        assert_eq!(joined, "Open File");
    }

    #[test]
    fn highlight_no_match_is_plain() {
        assert_eq!(
            highlight_runs("abc", "xyz"),
            vec![("abc".to_string(), false)]
        );
        assert_eq!(highlight_runs("abc", ""), vec![("abc".to_string(), false)]);
    }

    #[test]
    fn count_caps() {
        assert_eq!(count_label(7), "7");
        assert_eq!(count_label(100), "99+");
    }
}
