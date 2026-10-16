//! [`TextInput`]: a real editable text entity (single-line, or multi-line as a
//! textarea) built on gpui's input-handler APIs: cursor, selection, clipboard,
//! IME, focus ring. gpui 0.2.2 ships no text widget, so this is it.
//!
//! ```ignore
//! let name = cx.new(|cx| TextInput::new(cx).placeholder("handle…"));
//! let notes = cx.new(|cx| TextInput::new(cx).multiline(4));
//! div().child(name.clone())
//! ```
//! Key bindings are registered by [`crate::install`] (context `LgTextInput`).

use crate::paint::shaped;
use crate::panel::input_style;
use crate::text::first_family;
use crate::{hsla, theme_of};
use gpui::{
    actions, div, fill, point, prelude::*, px, relative, size, App, Bounds, ClipboardItem, Context,
    CursorStyle, ElementId, ElementInputHandler, Entity, EntityInputHandler, FocusHandle,
    Focusable, GlobalElementId, KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, PaintQuad, Pixels, Point, ShapedLine, SharedString, Style, TextRun,
    UTF16Selection, Window,
};
use std::ops::Range;
use std::rc::Rc;

actions!(
    dodai_text_input,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Up,
        Down,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        Enter,
        Paste,
        Cut,
        Copy
    ]
);

/// Register text-input key bindings (called by [`crate::install`]).
pub fn bind_keys(cx: &mut App) {
    let m = if cfg!(target_os = "macos") {
        "cmd"
    } else {
        "ctrl"
    };
    let ctx = Some("LgTextInput");
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, ctx),
        KeyBinding::new("delete", Delete, ctx),
        KeyBinding::new("left", Left, ctx),
        KeyBinding::new("right", Right, ctx),
        KeyBinding::new("up", Up, ctx),
        KeyBinding::new("down", Down, ctx),
        KeyBinding::new("shift-left", SelectLeft, ctx),
        KeyBinding::new("shift-right", SelectRight, ctx),
        KeyBinding::new("home", Home, ctx),
        KeyBinding::new("end", End, ctx),
        KeyBinding::new("enter", Enter, ctx),
        KeyBinding::new(&format!("{m}-a"), SelectAll, ctx),
        KeyBinding::new(&format!("{m}-v"), Paste, ctx),
        KeyBinding::new(&format!("{m}-c"), Copy, ctx),
        KeyBinding::new(&format!("{m}-x"), Cut, ctx),
    ]);
}

// ---- pure text math (byte offsets into `&str`) -------------------------------

/// Offset of the character boundary before `off` (0 at the start).
pub fn prev_boundary(s: &str, off: usize) -> usize {
    s[..off.min(s.len())]
        .char_indices()
        .next_back()
        .map_or(0, |(i, _)| i)
}

/// Offset of the character boundary after `off` (`len` at the end).
pub fn next_boundary(s: &str, off: usize) -> usize {
    let off = off.min(s.len());
    s[off..]
        .chars()
        .next()
        .map_or(s.len(), |c| off + c.len_utf8())
}

/// `(line index, byte column)` of `off`.
pub fn line_col(s: &str, off: usize) -> (usize, usize) {
    let before = &s[..off.min(s.len())];
    let line = before.matches('\n').count();
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    (line, before.len() - start)
}

/// `(start offset, text)` of every line (`'\n'` separated; always ≥ 1 line).
pub fn lines_of(s: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = 0;
    for l in s.split('\n') {
        out.push((start, l));
        start += l.len() + 1;
    }
    out
}

/// Cursor offset after moving one line up (`dir < 0`) or down, keeping the
/// character column. Off the first line → start of text; off the last → end.
pub fn vertical(s: &str, off: usize, dir: i32) -> usize {
    let lines = lines_of(s);
    let (li, col_bytes) = line_col(s, off);
    let (start, text) = lines[li];
    let col = text[..col_bytes.min(text.len())].chars().count();
    let target = li as i64 + dir.signum() as i64;
    if target < 0 {
        return 0;
    }
    if target as usize >= lines.len() {
        return s.len();
    }
    let (ts, tt) = lines[target as usize];
    let _ = start;
    ts + tt.char_indices().nth(col).map_or(tt.len(), |(i, _)| i)
}

type ChangeHandler = Rc<dyn Fn(&str, &mut Window, &mut App)>;
type SubmitHandler = Rc<dyn Fn(&str, &mut Window, &mut App)>;

struct Layout {
    lines: Vec<(usize, ShapedLine)>,
    bounds: Bounds<Pixels>,
    line_height: Pixels,
}

/// Editable text entity. Create with `cx.new(|cx| TextInput::new(cx))`.
pub struct TextInput {
    focus: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected: Range<usize>,
    reversed: bool,
    marked: Option<Range<usize>>,
    layout: Option<Layout>,
    selecting: bool,
    rows: Option<usize>,
    disabled: bool,
    on_change: Option<ChangeHandler>,
    on_submit: Option<SubmitHandler>,
}

impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle(),
            content: SharedString::default(),
            placeholder: SharedString::default(),
            selected: 0..0,
            reversed: false,
            marked: None,
            layout: None,
            selecting: false,
            rows: None,
            disabled: false,
            on_change: None,
            on_submit: None,
        }
    }
    /// Initial text (cursor at the end).
    pub fn value(mut self, v: impl Into<SharedString>) -> Self {
        self.content = v.into();
        let n = self.content.len();
        self.selected = n..n;
        self
    }
    /// Dimmed hint shown while empty.
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = p.into();
        self
    }
    /// Textarea: Enter inserts a newline and the box is `rows` lines tall.
    pub fn multiline(mut self, rows: usize) -> Self {
        self.rows = Some(rows.max(1));
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Called with the full text after every edit.
    pub fn on_change(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
    /// Called on Enter (single-line only).
    pub fn on_submit(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_submit = Some(Rc::new(f));
        self
    }
    /// Current text.
    pub fn text(&self) -> &str {
        &self.content
    }
    /// Replace the text programmatically.
    pub fn set_text(&mut self, v: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.content = v.into();
        let n = self.content.len();
        self.selected = n..n;
        self.marked = None;
        cx.notify();
    }

    fn multiline_on(&self) -> bool {
        self.rows.is_some()
    }
    fn cursor(&self) -> usize {
        if self.reversed {
            self.selected.start
        } else {
            self.selected.end
        }
    }
    fn move_to(&mut self, o: usize, cx: &mut Context<Self>) {
        self.selected = o..o;
        self.reversed = false;
        cx.notify();
    }
    fn select_to(&mut self, o: usize, cx: &mut Context<Self>) {
        if self.reversed {
            self.selected.start = o;
        } else {
            self.selected.end = o;
        }
        if self.selected.end < self.selected.start {
            self.reversed = !self.reversed;
            self.selected = self.selected.end..self.selected.start;
        }
        cx.notify();
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.move_to(prev_boundary(&self.content, self.cursor()), cx);
        } else {
            self.move_to(self.selected.start, cx);
        }
    }
    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.move_to(next_boundary(&self.content, self.selected.end), cx);
        } else {
            self.move_to(self.selected.end, cx);
        }
    }
    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(vertical(&self.content, self.cursor(), -1), cx);
    }
    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(vertical(&self.content, self.cursor(), 1), cx);
    }
    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(prev_boundary(&self.content, self.cursor()), cx);
    }
    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(next_boundary(&self.content, self.cursor()), cx);
    }
    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.selected = 0..self.content.len();
        self.reversed = false;
        cx.notify();
    }
    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        let (l, _) = line_col(&self.content, self.cursor());
        self.move_to(lines_of(&self.content)[l].0, cx);
    }
    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        let (l, _) = line_col(&self.content, self.cursor());
        let (s, t) = lines_of(&self.content)[l];
        self.move_to(s + t.len(), cx);
    }
    fn backspace(&mut self, _: &Backspace, w: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.select_to(prev_boundary(&self.content, self.cursor()), cx);
        }
        self.replace_text_in_range(None, "", w, cx);
    }
    fn delete(&mut self, _: &Delete, w: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_empty() {
            self.select_to(next_boundary(&self.content, self.cursor()), cx);
        }
        self.replace_text_in_range(None, "", w, cx);
    }
    fn enter(&mut self, _: &Enter, w: &mut Window, cx: &mut Context<Self>) {
        if self.multiline_on() {
            self.replace_text_in_range(None, "\n", w, cx);
        } else if let Some(f) = self.on_submit.clone() {
            f(&self.content.clone(), w, cx);
        }
    }
    fn paste(&mut self, _: &Paste, w: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = cx.read_from_clipboard().and_then(|i| i.text()) {
            let text = if self.multiline_on() {
                text
            } else {
                text.replace('\n', " ")
            };
            self.replace_text_in_range(None, &text, w, cx);
        }
    }
    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if !self.selected.is_empty() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[self.selected.clone()].to_string(),
            ));
        }
    }
    fn cut(&mut self, _: &Cut, w: &mut Window, cx: &mut Context<Self>) {
        if !self.selected.is_empty() {
            self.copy(&Copy, w, cx);
            self.replace_text_in_range(None, "", w, cx);
        }
    }

    fn on_mouse_down(&mut self, e: &MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.selecting = true;
        let i = self.index_for_position(e.position);
        if e.modifiers.shift {
            self.select_to(i, cx);
        } else {
            self.move_to(i, cx);
        }
    }
    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
    }
    fn on_mouse_move(&mut self, e: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.selecting {
            self.select_to(self.index_for_position(e.position), cx);
        }
    }

    fn index_for_position(&self, p: Point<Pixels>) -> usize {
        let Some(l) = self.layout.as_ref() else {
            return 0;
        };
        if self.content.is_empty() || l.lines.is_empty() {
            return 0;
        }
        let rel: f32 = ((p.y - l.bounds.top()) / l.line_height).floor();
        let i = (rel.max(0.0) as usize).min(l.lines.len() - 1);
        let (start, line) = &l.lines[i];
        start + line.closest_index_for_x(p.x - l.bounds.left())
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let (mut u8o, mut u16o) = (0, 0);
        for ch in self.content.chars() {
            if u16o >= offset {
                break;
            }
            u16o += ch.len_utf16();
            u8o += ch.len_utf8();
        }
        u8o
    }
    fn offset_to_utf16(&self, offset: usize) -> usize {
        let (mut u8o, mut u16o) = (0, 0);
        for ch in self.content.chars() {
            if u8o >= offset {
                break;
            }
            u8o += ch.len_utf8();
            u16o += ch.len_utf16();
        }
        u16o
    }
    fn range_to_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(r.start)..self.offset_to_utf16(r.end)
    }
    fn range_from_utf16(&self, r: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(r.start)..self.offset_from_utf16(r.end)
    }
    fn changed(&self, w: &mut Window, cx: &mut Context<Self>) {
        if let Some(f) = self.on_change.clone() {
            f(&self.content.clone(), w, cx);
        }
    }
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        r: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.range_from_utf16(&r);
        actual.replace(self.range_to_utf16(&r));
        Some(self.content[r].to_string())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected),
            reversed: self.reversed,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked.as_ref().map(|r| self.range_to_utf16(r))
    }
    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {
        self.marked = None;
    }
    fn replace_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        new: &str,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let r = r
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked.clone())
            .unwrap_or(self.selected.clone());
        self.content = format!(
            "{}{}{}",
            &self.content[..r.start],
            new,
            &self.content[r.end..]
        )
        .into();
        let c = r.start + new.len();
        self.selected = c..c;
        self.reversed = false;
        self.marked = None;
        self.changed(w, cx);
        cx.notify();
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        new: &str,
        sel: Option<Range<usize>>,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let r = r
            .as_ref()
            .map(|r| self.range_from_utf16(r))
            .or(self.marked.clone())
            .unwrap_or(self.selected.clone());
        self.content = format!(
            "{}{}{}",
            &self.content[..r.start],
            new,
            &self.content[r.end..]
        )
        .into();
        self.marked = (!new.is_empty()).then(|| r.start..r.start + new.len());
        self.selected = sel
            .as_ref()
            .map(|s| self.range_from_utf16(s))
            .map(|n| n.start + r.start..n.end + r.start)
            .unwrap_or_else(|| r.start + new.len()..r.start + new.len());
        self.changed(w, cx);
        cx.notify();
    }
    fn bounds_for_range(
        &mut self,
        r: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let l = self.layout.as_ref()?;
        let r = self.range_from_utf16(&r);
        let (li, col) = line_col(&self.content, r.start);
        let (_, line) = l.lines.get(li)?;
        let x = line.x_for_index(col);
        let top = bounds.top() + l.line_height * li as f32;
        Some(Bounds::new(
            point(bounds.left() + x, top),
            size(px(1.0), l.line_height),
        ))
    }
    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        self.layout.as_ref()?;
        Some(self.offset_to_utf16(self.index_for_position(p)))
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

struct Prepaint {
    lines: Vec<(usize, ShapedLine)>,
    selection: Vec<PaintQuad>,
    cursor: Option<PaintQuad>,
}

struct TextElement {
    input: Entity<TextInput>,
}

impl IntoElement for TextElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = Prepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let n = self.input.read(cx).content.split('\n').count().max(1);
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = (window.line_height() * n as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> Prepaint {
        let t = theme_of(cx);
        let tokens = t.input(true);
        let input = self.input.read(cx);
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let lh = window.line_height();
        let empty = input.content.is_empty();
        let mut lines = Vec::new();
        if empty {
            let p = input.placeholder.clone();
            let run = TextRun {
                len: p.len(),
                font: style.font(),
                color: hsla(tokens.placeholder),
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let runs: &[TextRun] = if p.is_empty() {
                &[]
            } else {
                std::slice::from_ref(&run)
            };
            lines.push((0, window.text_system().shape_line(p, font_size, runs, None)));
        } else {
            let color = if input.disabled {
                hsla(t.palette.text_disabled)
            } else {
                hsla(tokens.text)
            };
            for (start, text) in lines_of(&input.content) {
                let run = TextRun {
                    len: text.len(),
                    font: style.font(),
                    color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let runs: &[TextRun] = if text.is_empty() {
                    &[]
                } else {
                    std::slice::from_ref(&run)
                };
                lines.push((
                    start,
                    window.text_system().shape_line(
                        SharedString::from(text.to_string()),
                        font_size,
                        runs,
                        None,
                    ),
                ));
            }
        }
        let mut selection = Vec::new();
        let mut cursor = None;
        if !input.disabled {
            let sel = input.selected.clone();
            if sel.is_empty() {
                let (li, col) = line_col(&input.content, sel.start);
                let x = if empty {
                    px(0.)
                } else {
                    lines[li].1.x_for_index(col)
                };
                cursor = Some(fill(
                    Bounds::new(
                        point(bounds.left() + x, bounds.top() + lh * li as f32),
                        size(px(2.), lh),
                    ),
                    hsla(t.palette.primary),
                ));
            } else {
                for (i, (start, line)) in lines.iter().enumerate() {
                    let end = start + input.content.split('\n').nth(i).map_or(0, str::len);
                    let (a, b) = (sel.start.max(*start), sel.end.min(end));
                    if a < b {
                        selection.push(fill(
                            Bounds::from_corners(
                                point(
                                    bounds.left() + line.x_for_index(a - start),
                                    bounds.top() + lh * i as f32,
                                ),
                                point(
                                    bounds.left() + line.x_for_index(b - start),
                                    bounds.top() + lh * (i + 1) as f32,
                                ),
                            ),
                            hsla(tokens.selection),
                        ));
                    }
                }
            }
        }
        Prepaint {
            lines,
            selection,
            cursor,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        pre: &mut Prepaint,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus = self.input.read(cx).focus.clone();
        window.handle_input(
            &focus,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        for q in pre.selection.drain(..) {
            window.paint_quad(q);
        }
        let lh = window.line_height();
        for (i, (_, line)) in pre.lines.iter().enumerate() {
            let _ = line.paint(
                point(bounds.origin.x, bounds.origin.y + lh * i as f32),
                lh,
                window,
                cx,
            );
        }
        if focus.is_focused(window) {
            if let Some(c) = pre.cursor.take() {
                window.paint_quad(c);
            }
        }
        let lines = std::mem::take(&mut pre.lines);
        self.input.update(cx, |i, _| {
            i.layout = Some(Layout {
                lines,
                bounds,
                line_height: lh,
            })
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme_of(cx);
        let focused = self.focus.is_focused(window) && !self.disabled;
        let lh = (t.typography.size_md * 1.45).round();
        let mut style = input_style(&t, focused);
        if self.disabled {
            style.fill = Some(hsla(t.palette.surface_raised));
            style.stroke = Some(hsla(t.palette.border));
            style.glow = None;
        }
        let mut d = shaped(&t.shape, style)
            .w_full()
            .key_context("LgTextInput")
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .line_height(px(lh))
            .text_color(hsla(t.input(true).text))
            .cursor(if self.disabled {
                CursorStyle::OperationNotAllowed
            } else {
                CursorStyle::IBeam
            });
        d = match self.rows {
            Some(rows) => d.min_h(px(lh * rows as f32 + t.spacing.sm * 2.0 + 4.0)),
            None => d.h(px(t.shape.control_height)).flex().items_center(),
        };
        if !self.disabled {
            d = d
                .track_focus(&self.focus)
                .on_action(cx.listener(Self::backspace))
                .on_action(cx.listener(Self::delete))
                .on_action(cx.listener(Self::left))
                .on_action(cx.listener(Self::right))
                .on_action(cx.listener(Self::up))
                .on_action(cx.listener(Self::down))
                .on_action(cx.listener(Self::select_left))
                .on_action(cx.listener(Self::select_right))
                .on_action(cx.listener(Self::select_all))
                .on_action(cx.listener(Self::home))
                .on_action(cx.listener(Self::end))
                .on_action(cx.listener(Self::enter))
                .on_action(cx.listener(Self::paste))
                .on_action(cx.listener(Self::cut))
                .on_action(cx.listener(Self::copy))
                .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
                .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
                .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
                .on_mouse_move(cx.listener(Self::on_mouse_move));
        }
        d.child(
            div()
                .w_full()
                .px(px(t.spacing.sm))
                .py(px(if self.rows.is_some() {
                    t.spacing.sm
                } else {
                    0.0
                }))
                .child(TextElement { input: cx.entity() }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_are_char_aware() {
        let s = "aé日b";
        assert_eq!(next_boundary(s, 0), 1);
        assert_eq!(next_boundary(s, 1), 3);
        assert_eq!(next_boundary(s, 3), 6);
        assert_eq!(next_boundary(s, 7), 7);
        assert_eq!(prev_boundary(s, 7), 6);
        assert_eq!(prev_boundary(s, 3), 1);
        assert_eq!(prev_boundary(s, 0), 0);
    }

    #[test]
    fn lines_and_columns() {
        let s = "ab\ncde\n\nf";
        let l = lines_of(s);
        assert_eq!(l, vec![(0, "ab"), (3, "cde"), (7, ""), (8, "f")]);
        assert_eq!(line_col(s, 0), (0, 0));
        assert_eq!(line_col(s, 5), (1, 2));
        assert_eq!(line_col(s, 7), (2, 0));
        assert_eq!(lines_of("").len(), 1);
    }

    #[test]
    fn vertical_keeps_column_and_clamps() {
        let s = "ab\ncde\n\nf";
        assert_eq!(vertical(s, 1, 1), 4); // col 1 → "cde"[1]
        assert_eq!(vertical(s, 5, -1), 2); // col 2 clamps to end of "ab"
        assert_eq!(vertical(s, 5, 1), 7); // empty line
        assert_eq!(vertical(s, 1, -1), 0);
        assert_eq!(vertical(s, 8, 1), s.len());
    }
}
