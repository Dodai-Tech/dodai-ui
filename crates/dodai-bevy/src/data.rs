//! Data display: lists, tables, timelines, scroll areas, carousels, markdown, images, panes and
//! live sample feeds.

#![allow(clippy::type_complexity)]

use std::sync::{Arc, Mutex};

use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use dodai_core::stream::RingBuffer;
use dodai_core::{CardVariant, Tone};

use crate::interact::{UiSelectGroup, UiSelectable};
use crate::kit::{kit, kit_pressable, Kit};
use crate::parts::{caption, column, glyph, row, wide, Glyph};
use crate::widgets::{divider, text, ControlPointer, TextRole, UiText};

type Spawner<'a> = ChildSpawner<'a>;

/// One row of a [`list`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ListRow {
    /// Title.
    pub title: String,
    /// Second line.
    pub detail: Option<String>,
    /// Small line above the title.
    pub overline: Option<String>,
    /// Text at the end of the row.
    pub trailing: Option<String>,
    /// Initials of a round avatar before the text.
    pub avatar: Option<String>,
    /// Starts selected.
    pub selected: bool,
}

impl ListRow {
    /// A row with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..default()
        }
    }

    /// With a second line.
    #[must_use]
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }

    /// With a line above the title.
    #[must_use]
    pub fn overline(mut self, d: impl Into<String>) -> Self {
        self.overline = Some(d.into());
        self
    }

    /// With text at the end.
    #[must_use]
    pub fn trailing(mut self, d: impl Into<String>) -> Self {
        self.trailing = Some(d.into());
        self
    }

    /// With an avatar of these initials.
    #[must_use]
    pub fn avatar(mut self, d: impl Into<String>) -> Self {
        self.avatar = Some(d.into());
        self
    }

    /// Starts selected.
    #[must_use]
    pub fn selected(mut self, on: bool) -> Self {
        self.selected = on;
        self
    }
}

fn list_row(r: ListRow, i: usize) -> impl Bundle {
    (
        kit_pressable(Kit::Row {
            selected: r.selected,
        }),
        UiSelectable(i),
        wide(row(12.0)),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            if let Some(a) = r.avatar {
                c.spawn(crate::components::avatar(a, 40.0));
            }
            c.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    ..default()
                },
                Children::spawn(SpawnWith(move |t: &mut Spawner| {
                    if let Some(o) = r.overline {
                        t.spawn(caption(o));
                    }
                    t.spawn(text(r.title, TextRole::Content));
                    if let Some(d) = r.detail {
                        t.spawn(text(d, TextRole::Muted));
                    }
                })),
            ));
            if let Some(t) = r.trailing {
                c.spawn(caption(t));
            }
        })),
    )
}

/// A list of rows separated by dividers; pressing a row selects it.
pub fn list(rows: Vec<ListRow>) -> impl Bundle {
    let start = rows.iter().position(|r| r.selected);
    (
        UiSelectGroup { selected: start },
        wide(column()),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for (i, r) in rows.into_iter().enumerate() {
                if i > 0 {
                    c.spawn(divider());
                }
                c.spawn(list_row(r, i));
            }
        })),
    )
}

/// A column of a [`table`] or [`data_table`].
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// Header text.
    pub title: String,
    /// Align the cells to the right (numbers).
    pub right: bool,
    /// Clicking the header sorts by it ([`data_table`]).
    pub sortable: bool,
    /// Relative width.
    pub weight: f32,
}

impl Column {
    /// A column.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            right: false,
            sortable: false,
            weight: 1.0,
        }
    }

    /// Right-aligned.
    #[must_use]
    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }

    /// Sortable by its header.
    #[must_use]
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }

    /// Relative width.
    #[must_use]
    pub fn weight(mut self, w: f32) -> Self {
        self.weight = w.max(0.1);
        self
    }
}

fn cell(s: String, col: &Column, role: TextRole) -> impl Bundle {
    (
        Node {
            flex_basis: Val::Px(0.0),
            flex_grow: col.weight,
            min_width: Val::Px(0.0),
            justify_content: if col.right {
                JustifyContent::FlexEnd
            } else {
                JustifyContent::FlexStart
            },
            overflow: Overflow::clip(),
            ..default()
        },
        children![text(s, role)],
    )
}

/// A static table: a header row and banded body rows.
pub fn table(columns: Vec<Column>, rows: Vec<Vec<String>>) -> impl Bundle {
    (
        wide(column()),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            let cols = columns.clone();
            c.spawn((kit(Kit::TableHead), wide(row(8.0))))
                .with_children(|h| {
                    for col in &cols {
                        h.spawn(cell(col.title.clone(), col, TextRole::Label));
                    }
                });
            for (i, r) in rows.into_iter().enumerate() {
                let cols = columns.clone();
                c.spawn((kit(Kit::TableRow(i % 2 == 1)), wide(row(8.0))))
                    .with_children(|rw| {
                        for (col, v) in cols.iter().zip(r) {
                            rw.spawn(cell(v, col, TextRole::Content));
                        }
                    });
            }
        })),
    )
}

/// The rows and sort of a [`data_table`].
#[derive(Component, Debug, Clone, PartialEq)]
pub struct UiDataTable {
    /// The unsorted rows.
    pub rows: Vec<Vec<String>>,
    /// `(column, ascending)`.
    pub sort: Option<(usize, bool)>,
}

impl UiDataTable {
    /// The row indices in display order: numeric columns sort by value, others by text.
    pub fn order(&self) -> Vec<usize> {
        let mut idx: Vec<usize> = (0..self.rows.len()).collect();
        if let Some((c, asc)) = self.sort {
            idx.sort_by(|a, b| {
                let (x, y) = (self.rows[*a].get(c), self.rows[*b].get(c));
                let ord = match (
                    x.and_then(|s| s.parse::<f64>().ok()),
                    y.and_then(|s| s.parse::<f64>().ok()),
                ) {
                    (Some(p), Some(q)) => p.total_cmp(&q),
                    _ => x.cmp(&y),
                };
                if asc {
                    ord
                } else {
                    ord.reverse()
                }
            });
        }
        idx
    }
}

/// The header of a column of a [`data_table`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DataHead {
    pub col: usize,
    pub sortable: bool,
}

/// The body cell at display row `row` of column `col`.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DataCell {
    pub row: usize,
    pub col: usize,
}

/// A table whose sortable headers sort the rows (a second press reverses; the header shows the
/// arrow).
pub fn data_table(columns: Vec<Column>, rows: Vec<Vec<String>>) -> impl Bundle {
    let data = UiDataTable {
        rows: rows.clone(),
        sort: None,
    };
    (
        data,
        wide(column()),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            let cols = columns.clone();
            c.spawn((kit(Kit::TableHead), wide(row(8.0))))
                .with_children(|h| {
                    for (i, col) in cols.iter().enumerate() {
                        let mut e = h.spawn((
                            cell(col.title.clone(), col, TextRole::Label),
                            ControlPointer::default(),
                        ));
                        if col.sortable {
                            e.insert(DataHead {
                                col: i,
                                sortable: true,
                            });
                        }
                    }
                });
            for (ri, r) in rows.into_iter().enumerate() {
                let cols = columns.clone();
                c.spawn((kit(Kit::TableRow(ri % 2 == 1)), wide(row(8.0))))
                    .with_children(|rw| {
                        for (ci, (col, v)) in cols.iter().zip(r).enumerate() {
                            rw.spawn((
                                cell(v, col, TextRole::Content),
                                DataCell { row: ri, col: ci },
                            ));
                        }
                    });
            }
        })),
    )
}

pub(crate) fn data_table_sort(
    heads: Query<(&DataHead, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    parents: Query<&ChildOf>,
    mut tables: Query<&mut UiDataTable>,
) {
    for (h, p, parent) in &heads {
        if *p != ControlPointer::Pressed || !h.sortable {
            continue;
        }
        // head cell -> header row -> table
        let Ok(row) = parents.get(parent.parent()) else {
            continue;
        };
        let Ok(mut t) = tables.get_mut(row.parent()) else {
            continue;
        };
        t.sort = match t.sort {
            Some((c, true)) if c == h.col => Some((h.col, false)),
            _ => Some((h.col, true)),
        };
    }
}

pub(crate) fn data_table_cells(
    tables: Query<(Entity, &UiDataTable), Changed<UiDataTable>>,
    parents: Query<&ChildOf>,
    mut cells: Query<(Entity, &DataCell, &Children)>,
    mut texts: Query<&mut UiText>,
) {
    for (table, t) in &tables {
        let order = t.order();
        for (e, c, kids) in &mut cells {
            // cell -> row -> table
            let Ok(r) = parents.get(e) else { continue };
            let Ok(tt) = parents.get(r.parent()) else {
                continue;
            };
            if tt.parent() != table {
                continue;
            }
            let Some(src) = order.get(c.row) else {
                continue;
            };
            let Some(v) = t.rows[*src].get(c.col) else {
                continue;
            };
            for k in kids.iter() {
                if let Ok(mut ut) = texts.get_mut(k) {
                    if ut.raw != *v {
                        ut.raw = v.clone();
                    }
                }
            }
        }
    }
}

/// One event of a [`timeline`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TimelineEvent {
    /// What happened.
    pub title: String,
    /// When.
    pub time: String,
    /// More text.
    pub detail: Option<String>,
    /// Marker colour.
    pub tone: Option<Tone>,
}

impl TimelineEvent {
    /// An event.
    pub fn new(title: impl Into<String>, time: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            time: time.into(),
            ..default()
        }
    }

    /// With more text.
    #[must_use]
    pub fn detail(mut self, d: impl Into<String>) -> Self {
        self.detail = Some(d.into());
        self
    }

    /// With a marker tone.
    #[must_use]
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = Some(t);
        self
    }
}

/// One event: a dot beside its text.
pub fn timeline_event(e: TimelineEvent) -> impl Bundle {
    (
        Node {
            column_gap: Val::Px(16.0),
            align_items: AlignItems::FlexStart,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                kit(Kit::Dot(e.tone)),
                Node {
                    width: Val::Px(12.0),
                    height: Val::Px(12.0),
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
            ));
            c.spawn((
                column(),
                Children::spawn(SpawnWith(move |t: &mut Spawner| {
                    t.spawn(text(e.title, TextRole::Content));
                    t.spawn(caption(e.time));
                    if let Some(d) = e.detail {
                        t.spawn(text(d, TextRole::Muted));
                    }
                })),
            ));
        })),
    )
}

/// Events one under the other.
pub fn timeline(events: Vec<TimelineEvent>) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for e in events {
                c.spawn(timeline_event(e));
            }
        })),
    )
}

/// A scrolling box of fixed `height`; the mouse wheel scrolls it while the pointer is over it.
#[derive(Component, Debug, Clone, Copy, Default)]
#[require(ScrollPosition, bevy::ui::RelativeCursorPosition)]
pub struct UiScrollArea;

/// A box of `height` px that scrolls its content vertically.
pub fn scroll_area(height: f32) -> impl Bundle {
    (
        UiScrollArea,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(height),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            ..default()
        },
    )
}

pub(crate) fn scroll_wheel(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut areas: Query<
        (
            &bevy::ui::RelativeCursorPosition,
            &mut ScrollPosition,
            &ComputedNode,
        ),
        With<UiScrollArea>,
    >,
) {
    use bevy::input::mouse::MouseScrollUnit;
    let dy: f32 = wheel
        .read()
        .map(|w| match w.unit {
            MouseScrollUnit::Line => -w.y * 28.0,
            MouseScrollUnit::Pixel => -w.y,
        })
        .sum();
    if dy == 0.0 {
        return;
    }
    for (rel, mut pos, node) in &mut areas {
        if !rel.cursor_over() {
            continue;
        }
        let max = ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.0);
        pos.y = (pos.y + dy).clamp(0.0, max);
    }
}

/// A scroll area that highlights the section the page is on: `sections` are the labels of a
/// navigation list that follows the scroll (see [`dodai_core::spy`]).
pub fn scroll_spy(sections: &[&str]) -> impl Bundle {
    let labels: Vec<String> = sections.iter().map(|s| (*s).to_string()).collect();
    (
        UiSelectGroup { selected: Some(0) },
        column(),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for (i, l) in labels.into_iter().enumerate() {
                c.spawn((
                    kit_pressable(Kit::Row { selected: i == 0 }),
                    UiSelectable(i),
                    children![text(l, TextRole::Control)],
                ));
            }
        })),
    )
}

/// Pages stacked on one another with one shown; the wheel (or the arrows) turns the page.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSnapStack {
    /// The shown page.
    pub page: usize,
    /// Number of pages.
    pub len: usize,
}

/// A stack of `pages` children, one visible; add the pages as children.
pub fn snap_stack(len: usize) -> impl Bundle {
    (
        UiSnapStack { page: 0, len },
        wide(column()),
        bevy::ui::RelativeCursorPosition::default(),
    )
}

pub(crate) fn snap_stacks(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut stacks: Query<(&mut UiSnapStack, &bevy::ui::RelativeCursorPosition)>,
) {
    let dy: f32 = wheel.read().map(|w| w.y).sum();
    if dy == 0.0 {
        return;
    }
    for (mut s, rel) in &mut stacks {
        if !rel.cursor_over() || s.len == 0 {
            continue;
        }
        let next = dodai_core::paging::step_index(
            Some(s.page),
            s.len,
            if dy < 0.0 { 1 } else { -1 },
            false,
        )
        .unwrap_or(s.page);
        if next != s.page {
            s.page = next;
        }
    }
}

pub(crate) fn snap_stack_pages(
    stacks: Query<(&UiSnapStack, &Children), Changed<UiSnapStack>>,
    mut nodes: Query<&mut Node>,
) {
    for (s, kids) in &stacks {
        for (i, k) in kids.iter().enumerate() {
            if let Ok(mut n) = nodes.get_mut(k) {
                let d = if i == s.page {
                    Display::Flex
                } else {
                    Display::None
                };
                if n.display != d {
                    n.display = d;
                }
            }
        }
    }
}

/// A row of items with arrows to move through them: `visible` items show at a time.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiCarousel {
    /// Index of the first shown item.
    pub start: usize,
    /// Items shown at once.
    pub visible: usize,
    /// Number of items.
    pub len: usize,
}

/// The arrow of a [`UiCarousel`]: `-1` previous, `1` next.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CarouselArrow(pub i32);

/// The strip of items of a [`UiCarousel`].
#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct CarouselStrip;

/// A carousel of `labels` cards, `visible` at a time, with previous and next arrows.
pub fn carousel(labels: &[&str], visible: usize) -> impl Bundle {
    let labels: Vec<String> = labels.iter().map(|s| (*s).to_string()).collect();
    let len = labels.len();
    (
        UiCarousel {
            start: 0,
            visible: visible.max(1),
            len,
        },
        wide(row(8.0)),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                kit_pressable(Kit::Chip { selected: false }),
                CarouselArrow(-1),
                children![glyph(Glyph::ChevronLeft, 18)],
            ));
            c.spawn((
                CarouselStrip,
                Node {
                    flex_grow: 1.0,
                    ..row(8.0)
                },
                Children::spawn(SpawnWith(move |s: &mut Spawner| {
                    for l in labels {
                        s.spawn((
                            kit(Kit::Card(CardVariant::Outlined)),
                            Node {
                                flex_grow: 1.0,
                                flex_basis: Val::Px(0.0),
                                justify_content: JustifyContent::Center,
                                min_height: Val::Px(72.0),
                                ..default()
                            },
                            children![text(l, TextRole::Content)],
                        ));
                    }
                })),
            ));
            c.spawn((
                kit_pressable(Kit::Chip { selected: false }),
                CarouselArrow(1),
                children![glyph(Glyph::ChevronRight, 18)],
            ));
        })),
    )
}

pub(crate) fn carousel_arrows(
    pressed: Query<(&CarouselArrow, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    mut carousels: Query<&mut UiCarousel>,
) {
    for (a, p, parent) in &pressed {
        if *p != ControlPointer::Pressed {
            continue;
        }
        if let Ok(mut c) = carousels.get_mut(parent.parent()) {
            let max = c.len.saturating_sub(c.visible);
            let next = (c.start as i32 + a.0).clamp(0, max as i32) as usize;
            if next != c.start {
                c.start = next;
            }
        }
    }
}

pub(crate) fn carousel_items(
    carousels: Query<(&UiCarousel, &Children), Changed<UiCarousel>>,
    strips: Query<&Children, With<CarouselStrip>>,
    mut nodes: Query<&mut Node>,
) {
    for (c, kids) in &carousels {
        for k in kids.iter() {
            let Ok(items) = strips.get(k) else { continue };
            for (i, item) in items.iter().enumerate() {
                if let Ok(mut n) = nodes.get_mut(item) {
                    let d = if i >= c.start && i < c.start + c.visible {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    if n.display != d {
                        n.display = d;
                    }
                }
            }
        }
    }
}

/// Markdown text drawn with the theme's text roles: headings, paragraphs, lists, quotes and
/// code (parsed by [`dodai_core::markdown`]).
pub fn markdown(src: &str) -> impl Bundle {
    use dodai_core::markdown::{parse_blocks, parse_inline, MdBlock};
    let blocks = parse_blocks(src);
    (
        wide(column()),
        Node {
            row_gap: Val::Px(8.0),
            ..wide(column())
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for b in blocks {
                match b {
                    MdBlock::Heading(_, t) => {
                        c.spawn(text(parse_inline(&t).0, TextRole::Heading));
                    }
                    MdBlock::Paragraph(t) => {
                        c.spawn(text(parse_inline(&t).0, TextRole::Body));
                    }
                    MdBlock::Code { text: t, .. } => {
                        c.spawn(crate::components::code_block(t));
                    }
                    MdBlock::Quote(t) => {
                        c.spawn((
                            kit(Kit::Card(CardVariant::Outlined)),
                            children![text(parse_inline(&t).0, TextRole::Muted)],
                        ));
                    }
                    MdBlock::List(items) => {
                        let markers = dodai_core::markdown::list_markers(&items);
                        for (m, it) in markers.into_iter().zip(items) {
                            c.spawn((
                                text(format!("{m} {}", parse_inline(&it.text).0), TextRole::Body),
                                Node {
                                    margin: UiRect::left(Val::Px(16.0 * f32::from(it.depth))),
                                    ..default()
                                },
                            ));
                        }
                    }
                    MdBlock::Rule => {
                        c.spawn(divider());
                    }
                    _ => {}
                }
            }
        })),
    )
}

/// A picture: the loaded image, or a placeholder box while `handle` is `None`.
pub fn image(handle: Option<Handle<Image>>, w: f32, h: f32) -> impl Bundle {
    let node = Node {
        width: Val::Px(w),
        height: Val::Px(h),
        ..default()
    };
    match handle {
        Some(i) => (node, ImageNode::new(i), kit(Kit::Skeleton)),
        None => (node, ImageNode::default(), kit(Kit::Skeleton)),
    }
}

/// Two panes with a draggable divider between them; spawn the two panes as children.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiSplitPane {
    /// The first pane's share of the width, `0.1..=0.9`.
    pub fraction: f32,
}

/// The draggable divider of a [`UiSplitPane`].
#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct SplitHandle;

/// Two panes side by side with a handle between; the first child pane starts at `fraction`.
pub fn split_pane(fraction: f32) -> impl Bundle {
    (
        UiSplitPane {
            fraction: fraction.clamp(0.1, 0.9),
        },
        wide(Node::default()),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    )
}

/// The handle to put between the two panes of a [`split_pane`].
pub fn split_handle() -> impl Bundle {
    (
        SplitHandle,
        kit(Kit::Dot(None)),
        ControlPointer::default(),
        Node {
            width: Val::Px(6.0),
            align_self: AlignSelf::Stretch,
            ..default()
        },
    )
}

pub(crate) fn split_panes(
    handles: Query<(&ControlPointer, &ChildOf), With<SplitHandle>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut panes: Query<(&mut UiSplitPane, &Children, &ComputedNode, &UiTransform)>,
    mut nodes: Query<&mut Node>,
) {
    let Ok(w) = windows.single() else { return };
    let Some(cursor) = w.cursor_position() else {
        return;
    };
    for (h, parent) in &handles {
        if *h != ControlPointer::Pressed {
            continue;
        }
        if let Ok((mut s, kids, node, _)) = panes.get_mut(parent.parent()) {
            let width = node.size().x * node.inverse_scale_factor();
            if width > 0.0 {
                let f = (cursor.x / width).clamp(0.1, 0.9);
                if (f - s.fraction).abs() > 0.001 {
                    s.fraction = f;
                }
            }
            let _ = kids;
        }
    }
    for (s, kids, _, _) in &panes {
        if let Some(first) = kids.first() {
            if let Ok(mut n) = nodes.get_mut(*first) {
                let wd = Val::Percent(s.fraction * 100.0);
                if n.width != wd {
                    n.width = wd;
                }
            }
        }
    }
}

/// Samples pushed from any thread; clones share the queue. Attach it to a [`Live`] window.
#[derive(Clone, Debug, Default)]
pub struct Feed {
    queue: Arc<Mutex<Vec<f32>>>,
}

impl Feed {
    /// A new empty feed.
    pub fn new() -> Self {
        Self::default()
    }

    /// Queues a sample.
    pub fn push(&self, v: f32) {
        if let Ok(mut q) = self.queue.lock() {
            q.push(v);
        }
    }

    /// Takes the queued samples.
    pub fn drain(&self) -> Vec<f32> {
        self.queue
            .lock()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default()
    }
}

/// A fixed-size window of recent samples, filled from a [`Feed`] each frame.
#[derive(Component, Debug, Clone)]
pub struct Live {
    /// The window.
    pub window: RingBuffer,
    feed: Option<Feed>,
}

impl Live {
    /// A window of `capacity` samples.
    pub fn new(capacity: usize) -> Self {
        Self {
            window: RingBuffer::new(capacity),
            feed: None,
        }
    }

    /// Reads from `feed` from now on.
    #[must_use]
    pub fn attach(mut self, feed: &Feed) -> Self {
        self.feed = Some(feed.clone());
        self
    }

    /// The samples, oldest first.
    pub fn samples(&self) -> Vec<f32> {
        self.window.to_vec()
    }
}

/// A [`Live`] window as a component: `live(64).attach(&feed)` on an entity.
pub fn live(capacity: usize) -> Live {
    Live::new(capacity)
}

pub(crate) fn drain_feeds(mut q: Query<&mut Live>) {
    for mut l in &mut q {
        let Some(f) = l.feed.clone() else { continue };
        for v in f.drain() {
            l.window.push(v);
        }
    }
}

/// The page grid of the design at a width: columns, gap and margin ([`dodai_core::GridSpec`]).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiPageGrid {
    /// The width to lay out for; `None` follows the window.
    pub width: Option<f32>,
}

/// A wrapping grid: children size themselves with [`UiGridSpan`] (columns of the design grid).
pub fn page_grid() -> impl Bundle {
    (
        UiPageGrid { width: None },
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },
    )
}

/// A cell of a [`page_grid`] spanning `0` columns of the grid.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiGridSpan(pub u32);

pub(crate) fn page_grids(
    theme: Res<crate::theme::DodaiTheme>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut grids: Query<(&UiPageGrid, &Children, &mut Node), Without<UiGridSpan>>,
    mut cells: Query<(&UiGridSpan, &mut Node)>,
) {
    let win = windows.single().map(Window::width).unwrap_or(1280.0);
    for (g, kids, mut n) in &mut grids {
        let width = g.width.unwrap_or(win);
        let spec = theme.page_grid(width);
        let gap = Val::Px(spec.gap);
        if n.column_gap != gap {
            n.column_gap = gap;
            n.row_gap = gap;
        }
        let pad = UiRect::horizontal(Val::Px(spec.margin));
        if n.padding != pad {
            n.padding = pad;
        }
        if let Some(m) = spec.max_width {
            n.max_width = Val::Px(m);
        }
        let inner = (width - 2.0 * spec.margin).min(spec.max_width.unwrap_or(f32::MAX));
        let cols = spec.columns.max(1) as f32;
        let col_w = (inner - spec.gap * (cols - 1.0)) / cols;
        for k in kids.iter() {
            if let Ok((s, mut c)) = cells.get_mut(k) {
                let span = s.0.clamp(1, spec.columns.max(1)) as f32;
                let w = Val::Px(col_w * span + spec.gap * (span - 1.0));
                if c.width != w {
                    c.width = w;
                }
            }
        }
    }
}

/// Registers the systems above.
pub(crate) fn add(app: &mut App) {
    app.add_message::<bevy::input::mouse::MouseWheel>()
        .add_systems(
            Update,
            (
                data_table_sort,
                data_table_cells,
                scroll_wheel,
                snap_stacks,
                snap_stack_pages,
                carousel_arrows,
                carousel_items,
                split_panes,
                drain_feeds,
                page_grids,
            )
                .chain()
                .after(crate::kit::press_input),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        crate::testing::component_app()
    }

    #[test]
    fn keys_move_a_hovered_slider_and_carousel() {
        use crate::widgets::UiSlider;
        use bevy::input::keyboard::{Key, KeyboardInput};
        let mut app = app();
        let sl = app
            .world_mut()
            .spawn(crate::widgets::slider(0.0, 0.0, 10.0))
            .id();
        let ca = app
            .world_mut()
            .spawn(carousel(&["a", "b", "c", "d"], 2))
            .id();
        app.update();
        *app.world_mut().get_mut::<ControlPointer>(sl).unwrap() = ControlPointer::Hovered;
        let arrow = app.world().get::<Children>(ca).unwrap()[0];
        *app.world_mut().get_mut::<ControlPointer>(arrow).unwrap() = ControlPointer::Hovered;
        for k in [Key::End, Key::ArrowLeft] {
            app.world_mut().write_message(KeyboardInput {
                key_code: KeyCode::Unidentified(bevy::input::keyboard::NativeKeyCode::Unidentified),
                logical_key: k,
                state: bevy::input::ButtonState::Pressed,
                text: None,
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
            app.update();
        }
        assert_eq!(
            app.world().get::<UiSlider>(sl).unwrap().value,
            9.5,
            "End, then a step back"
        );
        assert_eq!(
            app.world().get::<UiCarousel>(ca).unwrap().start,
            1,
            "End stops at the last page, Left steps back"
        );
    }

    #[test]
    fn data_table_order_sorts_numbers_by_value_and_text_by_text() {
        let mut t = UiDataTable {
            rows: vec![
                vec!["b".into(), "10".into()],
                vec!["a".into(), "9".into()],
                vec!["c".into(), "100".into()],
            ],
            sort: Some((1, true)),
        };
        assert_eq!(t.order(), vec![1, 0, 2]);
        t.sort = Some((1, false));
        assert_eq!(t.order(), vec![2, 0, 1]);
        t.sort = Some((0, true));
        assert_eq!(t.order(), vec![1, 0, 2]);
        t.sort = None;
        assert_eq!(t.order(), vec![0, 1, 2]);
    }

    #[test]
    fn pressing_a_sortable_header_reorders_the_cells() {
        let mut app = app();
        let cols = vec![
            Column::new("Name").sortable(),
            Column::new("N").right().sortable(),
        ];
        let rows = vec![
            vec!["b".to_string(), "2".to_string()],
            vec!["a".to_string(), "1".to_string()],
        ];
        let t = app.world_mut().spawn(data_table(cols, rows)).id();
        app.update();
        let head_row = app.world().get::<Children>(t).unwrap()[0];
        let first_head = app.world().get::<Children>(head_row).unwrap()[0];
        *app.world_mut()
            .get_mut::<ControlPointer>(first_head)
            .unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(
            app.world().get::<UiDataTable>(t).unwrap().sort,
            Some((0, true))
        );
        let first_cell_text = {
            let mut q = app.world_mut().query::<(&DataCell, &Children)>();
            let (_, kids) = q
                .iter(app.world())
                .find(|(c, _)| c.row == 0 && c.col == 0)
                .unwrap();
            app.world().get::<UiText>(kids[0]).unwrap().raw.clone()
        };
        assert_eq!(first_cell_text, "a");
    }

    #[test]
    fn carousel_arrows_move_the_window_and_stop_at_the_ends() {
        let mut app = app();
        let c = app
            .world_mut()
            .spawn(carousel(&["1", "2", "3", "4"], 2))
            .id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(c).unwrap().iter().collect();
        let (prev, next) = (kids[0], kids[2]);
        *app.world_mut().get_mut::<ControlPointer>(prev).unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(app.world().get::<UiCarousel>(c).unwrap().start, 0);
        for want in [1, 2, 2] {
            *app.world_mut().get_mut::<ControlPointer>(next).unwrap() = ControlPointer::Hovered;
            *app.world_mut().get_mut::<ControlPointer>(next).unwrap() = ControlPointer::Pressed;
            app.update();
            assert_eq!(app.world().get::<UiCarousel>(c).unwrap().start, want);
        }
        let strip = kids[1];
        let items: Vec<Entity> = app.world().get::<Children>(strip).unwrap().iter().collect();
        let shown: Vec<bool> = items
            .iter()
            .map(|i| app.world().get::<Node>(*i).unwrap().display == Display::Flex)
            .collect();
        assert_eq!(shown, vec![false, false, true, true]);
    }

    #[test]
    fn a_feed_fills_a_live_window_from_any_thread() {
        let mut app = app();
        let feed = Feed::new();
        let e = app.world_mut().spawn(live(3).attach(&feed)).id();
        let f2 = feed.clone();
        std::thread::spawn(move || {
            for i in 0..5 {
                f2.push(i as f32);
            }
        })
        .join()
        .unwrap();
        app.update();
        assert_eq!(
            app.world().get::<Live>(e).unwrap().samples(),
            vec![2.0, 3.0, 4.0]
        );
    }

    #[test]
    fn page_grid_cells_span_the_design_columns() {
        let mut app = app();
        let g = app.world_mut().spawn(page_grid()).id();
        let a = app.world_mut().spawn((UiGridSpan(1), Node::default())).id();
        let b = app.world_mut().spawn((UiGridSpan(2), Node::default())).id();
        app.world_mut().entity_mut(g).add_children(&[a, b]);
        app.world_mut().get_mut::<UiPageGrid>(g).unwrap().width = Some(1200.0);
        app.update();
        let w = |e: Entity| match app.world().get::<Node>(e).unwrap().width {
            Val::Px(p) => p,
            _ => panic!("px"),
        };
        assert!(w(b) > w(a), "{} > {}", w(b), w(a));
    }

    #[test]
    fn list_and_table_build_their_rows() {
        let mut app = app();
        let l = app
            .world_mut()
            .spawn(list(vec![
                ListRow::new("a").selected(true),
                ListRow::new("b").detail("d"),
            ]))
            .id();
        let t = app
            .world_mut()
            .spawn(table(
                vec![Column::new("X")],
                vec![vec!["1".into()], vec!["2".into()]],
            ))
            .id();
        app.update();
        // rows + one divider
        assert_eq!(app.world().get::<Children>(l).unwrap().len(), 3);
        assert_eq!(
            app.world().get::<UiSelectGroup>(l).unwrap().selected,
            Some(0)
        );
        // head + 2 rows
        assert_eq!(app.world().get::<Children>(t).unwrap().len(), 3);
    }

    #[test]
    fn timeline_lists_every_event() {
        let mut app = app();
        let t = app
            .world_mut()
            .spawn(timeline(vec![
                TimelineEvent::new("a", "1").tone(Tone::Success),
                TimelineEvent::new("b", "2"),
            ]))
            .id();
        app.update();
        assert_eq!(app.world().get::<Children>(t).unwrap().len(), 2);
    }
}
