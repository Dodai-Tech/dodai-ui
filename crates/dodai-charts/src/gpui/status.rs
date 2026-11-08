//! Status charts for operations dashboards: [`SlotGrid`] (which slots of a band are in use,
//! open, restricted or excluded), [`StatusStrip`] (health over time), [`BucketBar`] (counts in
//! graded buckets), [`StageBars`] (success per stage of a flow) and [`GradientScale`] (a
//! reading on a bad-to-good scale).

use crate::gpui::common::{fmt_with, label_column, label_style, nice_ticks, norm, Legend, NumFmt};
use crate::gpui::focus::{readout, Focus};
use dodai_core::{Color, Theme, Tone};
use dodai_gpui::paint::{shaped, ShapeStyle};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, relative, App, Bounds, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// A bar, cell or track in the design's own shape (corners, chamfers, outline, hard shadow),
/// like every chart mark: `fill` with the design's mark outline, or `edge` when given.
fn mark(t: &Theme, fill: Color, edge: Option<Color>) -> gpui::Div {
    let outline = edge.or(t.chart_style().mark_outline);
    shaped(
        &t.shape,
        ShapeStyle::from_tokens(t, Some(fill), outline, false, false),
    )
}

/// A thin mark (a rail, a strip) in the design's shape but never outlined: at a few px an
/// outline would swallow the colour.
fn thin(t: &Theme, fill: Color) -> gpui::Div {
    let mut style = ShapeStyle::from_tokens(t, Some(fill), None, false, false);
    style.stroke = None;
    style.stroke_width = 0.0;
    shaped(&t.shape, style)
}

/// `c` faded towards the surface: the quiet form of a status colour.
fn quiet(t: &Theme, c: Color) -> Color {
    c.mix(t.palette.surface, 0.7)
}

/// The colour `f` of the way from bad (0) through warning to good (1).
fn bad_to_good(t: &Theme, f: f32) -> Color {
    let f = if f.is_finite() {
        f.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let (bad, mid, good) = (
        t.palette.tone(Tone::Danger),
        t.palette.tone(Tone::Warning),
        t.palette.tone(Tone::Success),
    );
    if f < 0.5 {
        bad.mix(mid, f * 2.0)
    } else {
        mid.mix(good, (f - 0.5) * 2.0)
    }
}

// ─────────────────────────────── SlotGrid ───────────────────────────────

/// The state of one block of a [`SlotGrid`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotState {
    /// In use right now.
    Active,
    /// Free to use.
    Open,
    /// Usable with restrictions (drawn hatched).
    Restricted,
    /// Cannot be used here.
    Unavailable,
    /// Switched off by the user.
    Excluded,
}

impl SlotState {
    /// Every state, in legend order.
    pub const ALL: [Self; 5] = [
        Self::Active,
        Self::Open,
        Self::Restricted,
        Self::Unavailable,
        Self::Excluded,
    ];

    /// A generic legend name (see [`SlotGrid::legend`]).
    pub fn label(self) -> &'static str {
        match self {
            Self::Active => "In use",
            Self::Open => "Open",
            Self::Restricted => "Restricted",
            Self::Unavailable => "Unavailable",
            Self::Excluded => "Excluded",
        }
    }

    /// `(fill, edge)` of a block in this state.
    fn colours(self, t: &Theme) -> (Color, Color) {
        let tone = |k| t.palette.tone(k);
        match self {
            Self::Active => (tone(Tone::Success), tone(Tone::Success)),
            Self::Open | Self::Restricted => (t.palette.surface_raised, t.palette.border_strong),
            Self::Unavailable => (quiet(t, tone(Tone::Warning)), tone(Tone::Warning)),
            Self::Excluded => (quiet(t, tone(Tone::Danger)), quiet(t, tone(Tone::Danger))),
        }
    }
}

/// One block of a [`SlotGrid`] row: `span` slots from slot `start` (fractions allowed).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    /// First slot covered.
    pub start: f32,
    /// How many slots it covers.
    pub span: f32,
    /// What state it is in.
    pub state: SlotState,
}

impl Slot {
    /// A block covering `span` slots from `start`.
    pub fn new(start: f32, span: f32, state: SlotState) -> Self {
        Self { start, span, state }
    }
}

/// Picks a block: `(row, block)`.
type Pick = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;

/// A band of numbered slots and, per row, the blocks laid over them: which channels a radio
/// uses at each width, which seats or time slots are taken. Blocks are coloured by
/// [`SlotState`]; with [`SlotGrid::on_pick`] they are clickable (to exclude one, say).
/// [`SlotGrid::compact`] draws thin rails with the slot names underneath.
#[derive(IntoElement)]
pub struct SlotGrid {
    follow: dodai_gpui::follow::Follow,
    id: SharedString,
    caption: Option<(SharedString, SharedString)>,
    corner: Option<SharedString>,
    slots: Vec<SharedString>,
    rows: Vec<(SharedString, Vec<Slot>)>,
    compact: bool,
    legend: Option<[SharedString; 5]>,
    on_pick: Option<Pick>,
}

impl SlotGrid {
    dodai_gpui::follow_api!();
    /// A grid over the named `slots` (left to right).
    #[track_caller]
    pub fn new<S: Into<SharedString>>(slots: impl IntoIterator<Item = S>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            id: SharedString::default(),
            caption: None,
            corner: None,
            slots: slots.into_iter().map(Into::into).collect(),
            rows: Vec::new(),
            compact: false,
            legend: None,
            on_pick: None,
        }
    }
    /// A row of blocks.
    pub fn row(
        mut self,
        label: impl Into<SharedString>,
        blocks: impl IntoIterator<Item = Slot>,
    ) -> Self {
        self.rows.push((label.into(), blocks.into_iter().collect()));
        self
    }
    /// A title and a quieter note above the grid ("2.4 GHz", "2412–2484 MHz").
    pub fn caption(
        mut self,
        title: impl Into<SharedString>,
        note: impl Into<SharedString>,
    ) -> Self {
        self.caption = Some((title.into(), note.into()));
        self
    }
    /// Name of the slot header row, shown in the row-label column ("Channels").
    pub fn corner(mut self, name: impl Into<SharedString>) -> Self {
        self.corner = Some(name.into());
        self
    }
    /// Thin rails with the slot names underneath, for an overview.
    pub fn compact(mut self, on: bool) -> Self {
        self.compact = on;
        self
    }
    /// A legend underneath naming the states in [`SlotState::ALL`] order (the domain's own
    /// words; `SlotState::ALL.map(SlotState::label)` for generic ones).
    pub fn legend<S: Into<SharedString>>(mut self, names: [S; 5]) -> Self {
        self.legend = Some(names.map(Into::into));
        self
    }
    dodai_gpui::callback!(
        /// Called with `(row, block)` when a block is clicked.
        on_pick,
        usize,
        usize
    );
}

impl SlotGrid {
    fn block(&self, t: &Theme, r: usize, k: usize, b: Slot, h: f32) -> gpui::AnyElement {
        let n = self.slots.len().max(1) as f32;
        let (fill, edge) = b.state.colours(t);
        // A rail has no outline to show an open slot: it takes the edge colour instead.
        let fill = if self.compact && b.state != SlotState::Active {
            edge
        } else {
            fill
        };
        let gap = if self.compact {
            t.spacing.xs / 2.0
        } else {
            t.spacing.xs / 2.0 + 1.0
        };
        let outline = (!self.compact).then_some(edge);
        let mut cell = div()
            .id((SharedString::from(format!("{}-{r}", self.id)), k))
            .size_full()
            .relative()
            .child(
                if self.compact {
                    thin(t, fill)
                } else {
                    mark(t, fill, outline)
                }
                .absolute()
                .size_full(),
            );
        if b.state == SlotState::Restricted {
            // Inside the outline and clear of rounded or cut corners.
            let inset = t.shape.border_width.max(1.0) + t.shape.radius.min(t.spacing.xs) / 3.0;
            let c = hsla(t.palette.border_strong);
            cell = cell.child(
                canvas(
                    |_, _, _| (),
                    move |bb: Bounds<Pixels>, _, w, _| {
                        crate::gpui::common::hatch(
                            w,
                            bb.origin,
                            (bb.size.width.into(), bb.size.height.into()),
                            c,
                            crate::gpui::common::Hatch::Slash,
                        );
                    },
                )
                .absolute()
                .top(px(inset))
                .left(px(inset))
                .bottom(px(inset))
                .right(px(inset)),
            );
        }
        if let Some(f) = self.on_pick.clone() {
            cell = cell
                .cursor_pointer()
                .hover(|s| s.opacity(0.75))
                .on_click(move |_, window, cx| f(r, k, window, cx));
        }
        div()
            .absolute()
            .top_0()
            .h(px(h))
            .left(relative(b.start.max(0.0) / n))
            .w(relative(b.span.max(0.0) / n))
            .px(px(gap))
            .child(cell)
            .into_any_element()
    }

    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.id = self.follow.element_id(window, cx).to_string().into();
        let t = theme_of(cx);
        let gutter = if self.compact {
            0.0
        } else {
            label_column(
                &t,
                self.rows
                    .iter()
                    .map(|r| r.0.as_ref())
                    .chain(self.corner.as_ref().map(|c| c.as_ref())),
            )
        };
        let h = if self.compact {
            t.spacing.xs
        } else {
            t.sizes().chart_bar + t.spacing.xs
        };
        let names = |d: gpui::Div| {
            label_style(&t, d.flex().flex_1()).children(
                self.slots
                    .iter()
                    .map(|s| div().flex_1().flex().justify_center().child(s.clone())),
            )
        };
        let label_col = |s: Option<SharedString>| {
            label_style(&t, div())
                .w(px(gutter))
                .flex_none()
                .whitespace_nowrap()
                .children(s)
        };
        let mut out = div().flex().flex_col().gap(px(t.spacing.sm)).w_full();
        if let Some((title, note)) = self.caption.clone() {
            out = out.child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(t.spacing.md))
                    .child(div().text_color(hsla(t.palette.text)).child(title))
                    .child(label_style(&t, div()).child(note)),
            );
        }
        if !self.compact {
            out = out.child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .child(label_col(self.corner.clone()))
                    .child(names(div())),
            );
        }
        for (r, (label, blocks)) in self.rows.iter().enumerate() {
            let mut lane = div().relative().flex_1().h(px(h));
            // Blocks with no place or no width are left out (indices stay those of the row).
            for (k, b) in blocks.iter().enumerate() {
                if b.start.is_finite() && b.span.is_finite() && b.span > 0.0 {
                    lane = lane.child(self.block(&t, r, k, *b, h));
                }
            }
            let row = div().flex().items_center().gap(px(t.spacing.sm));
            out = out.child(if self.compact {
                row.child(lane)
            } else {
                row.child(label_col(Some(label.clone()))).child(lane)
            });
        }
        if self.compact {
            out = out.child(names(div()));
        }
        if let Some(names) = self.legend.clone() {
            out = out.child(Legend::new(
                SlotState::ALL
                    .into_iter()
                    .zip(names)
                    .map(|(s, n)| (n, s.colours(&t).1)),
            ));
        }
        out
    }
}

dodai_gpui::entry_render!(SlotGrid);

// ─────────────────────────────── StatusStrip ───────────────────────────────

/// Health over time as one thin strip: each span coloured by its [`Tone`], gaps left empty.
/// Sits under a time chart sharing its range.
#[derive(IntoElement)]
pub struct StatusStrip {
    follow: dodai_gpui::follow::Follow,
    range: (f32, f32),
    spans: Vec<(f32, f32, Tone)>,
    fmt: Option<NumFmt>,
}

/// What a tone means on a status strip.
fn tone_name(tone: Tone) -> &'static str {
    match tone {
        Tone::Info => "Info",
        Tone::Success => "Healthy",
        Tone::Warning => "Degraded",
        Tone::Danger => "Failing",
    }
}

impl StatusStrip {
    dodai_gpui::follow_api!();
    /// A strip over `lo..hi` with `(start, end, tone)` spans.
    #[track_caller]
    pub fn new(lo: f32, hi: f32, spans: impl IntoIterator<Item = (f32, f32, Tone)>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            range: (lo, hi),
            spans: spans.into_iter().collect(),
            fmt: None,
        }
    }
    /// Custom formatter for the span bounds in the readout.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.fmt = Some(Rc::new(f));
        self
    }

    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (lo, hi) = self.range;
        let finite = lo.is_finite() && hi.is_finite() && hi > lo;
        let focus = Focus::new(window, cx, &self.follow, self.spans.len());
        let key = SharedString::from(format!(
            "strip-{:?}",
            self.follow.sub("spans").element_id(window, cx)
        ));
        let mut strip = div()
            .relative()
            .w_full()
            .h(px(t.spacing.xs))
            .child(thin(&t, t.palette.border).absolute().size_full());
        for (i, (a, b, tone)) in self.spans.iter().enumerate() {
            let (fa, fb) = (norm(*a, lo, hi), norm(*b, lo, hi));
            if finite && fa.is_finite() && fb.is_finite() && fb > fa {
                let span = thin(&t, t.palette.tone(*tone))
                    .id((key.clone(), i))
                    .absolute()
                    .top_0()
                    .h_full()
                    .left(relative(fa))
                    .w(relative(fb - fa))
                    .opacity(focus.opacity(i, cx));
                strip = strip.child(focus.bind(span, i));
            }
        }
        let text = focus.shown(cx).and_then(|i| {
            let (a, b, tone) = self.spans.get(i)?;
            Some(format!(
                "{} · {} to {}",
                tone_name(*tone),
                fmt_with(&self.fmt, *a),
                fmt_with(&self.fmt, *b)
            ))
        });
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(strip)
            .child(readout(&t, text, "Hover a span"))
    }
}

dodai_gpui::entry_render!(StatusStrip);

// ─────────────────────────────── BucketBar ───────────────────────────────

/// Counts in graded buckets, worst on the left: every bucket tinted from bad to good, the ones
/// holding something in full colour with their count ("how many radios retry 5–10 %").
#[derive(IntoElement)]
pub struct BucketBar {
    follow: dodai_gpui::follow::Follow,
    edges: Vec<SharedString>,
    counts: Vec<u32>,
}

impl BucketBar {
    dodai_gpui::follow_api!();
    /// Buckets between `edges` (one more edge than counts), worst first.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(
        edges: impl IntoIterator<Item = S>,
        counts: impl IntoIterator<Item = u32>,
    ) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            edges: edges.into_iter().map(Into::into).collect(),
            counts: counts.into_iter().collect(),
        }
    }

    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let n = self.counts.len();
        let h = t.sizes().chart_bar;
        let focus = Focus::new(window, cx, &self.follow, n);
        let key = SharedString::from(format!(
            "buckets-{:?}",
            self.follow.sub("cells").element_id(window, cx)
        ));
        let cells = div().flex().gap(px(t.spacing.xs)).w_full().children(
            self.counts.iter().enumerate().map(|(i, c)| {
                let tone = bad_to_good(
                    &t,
                    if n > 1 {
                        i as f32 / (n - 1) as f32
                    } else {
                        1.0
                    },
                );
                let (bg, ink) = if *c > 0 {
                    (tone, t.on(tone))
                } else {
                    (tone.mix(t.palette.surface, 0.8), t.palette.text_muted)
                };
                focus.bind(
                    div()
                        .id((key.clone(), i))
                        .flex_1()
                        .h(px(h))
                        .relative()
                        .opacity(focus.opacity(i, cx))
                        .child(mark(&t, bg, None).absolute().size_full())
                        .child(
                            label_style(&t, div())
                                .absolute()
                                .size_full()
                                .text_color(hsla(ink))
                                .flex()
                                .items_center()
                                .justify_center()
                                .children((*c > 0).then(|| c.to_string())),
                        ),
                    i,
                )
            }),
        );
        let text = focus.shown(cx).and_then(|i| {
            let count = self.counts.get(i)?;
            let (a, b) = (self.edges.get(i)?, self.edges.get(i + 1)?);
            Some(format!("{a} to {b} · {count}"))
        });
        let edges = label_style(&t, div())
            .flex()
            .justify_between()
            .children(self.edges.iter().map(|e| div().child(e.clone())));
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(cells)
            .child(edges)
            .child(readout(&t, text, "Hover a bucket"))
    }
}

dodai_gpui::entry_render!(BucketBar);

// ─────────────────────────────── StageBars ───────────────────────────────

/// Success per stage of a flow (connect, sign in, get an address, resolve names): each stage a
/// bar filled to its success rate with the stage name on it and the rate underneath, coloured
/// by how healthy it is (good from 95 %, fading through amber to bad at 80 %).
#[derive(IntoElement)]
pub struct StageBars {
    follow: dodai_gpui::follow::Follow,
    stages: Vec<(SharedString, f32)>,
    fmt: Option<NumFmt>,
}

impl StageBars {
    dodai_gpui::follow_api!();
    /// Stages with their success rate in percent.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(stages: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            stages: stages.into_iter().map(|(s, v)| (s.into(), v)).collect(),
            fmt: None,
        }
    }
    /// Custom formatter for the rates (default "97%").
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.fmt = Some(Rc::new(f));
        self
    }

    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let target: Vec<f32> = self.stages.iter().map(|s| s.1).collect();
        let shown = dodai_gpui::follow::follow_all(window, cx, &self.follow, &target);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "StageBars");
        let h = t.sizes().chart_bar;
        let fmt = self
            .fmt
            .clone()
            .unwrap_or_else(|| Rc::new(|v| format!("{v:.0}%")));
        let focus = Focus::new(window, cx, &self.follow, self.stages.len());
        let key = SharedString::from(format!(
            "stages-{:?}",
            self.follow.sub("bars").element_id(window, cx)
        ));
        let text = focus.shown(cx).and_then(|i| {
            let (name, v) = self.stages.get(i)?;
            Some(format!(
                "{name} · {} succeeded",
                fmt_with(&Some(fmt.clone()), *v)
            ))
        });
        let row = div().flex().gap(px(t.spacing.xs)).w_full().children(
            self.stages
                .iter()
                .zip(shown)
                .enumerate()
                .map(|(i, ((name, _), v))| {
                    let v = if v.is_finite() {
                        v.clamp(0.0, 100.0)
                    } else {
                        0.0
                    };
                    // Healthy from 95 %, worrying below 80 %.
                    let tone = bad_to_good(&t, norm(v, 80.0, 95.0));
                    let stage = div()
                        .id((key.clone(), i))
                        .flex_1()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap(px(t.spacing.xs))
                        .opacity(focus.opacity(i, cx))
                        .child(
                            div()
                                .relative()
                                .h(px(h))
                                .child(
                                    mark(&t, tone.mix(t.palette.surface, 0.8), None)
                                        .absolute()
                                        .size_full(),
                                )
                                .child(
                                    mark(&t, tone, None)
                                        .absolute()
                                        .top_0()
                                        .left_0()
                                        .h_full()
                                        .w(relative(v / 100.0 * g)),
                                )
                                .child(
                                    label_style(&t, div())
                                        .absolute()
                                        .size_full()
                                        .px(px(t.spacing.xs))
                                        .flex()
                                        .items_center()
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .text_ellipsis()
                                        .text_color(hsla(t.on(tone)))
                                        .child(name.clone()),
                                ),
                        )
                        .child(
                            label_style(&t, div())
                                .text_color(hsla(tone))
                                .child(fmt_with(&Some(fmt.clone()), v)),
                        );
                    focus.bind(stage, i)
                }),
        );
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(row)
            .child(readout(&t, text, "Hover a stage"))
    }
}

dodai_gpui::entry_render!(StageBars);

// ─────────────────────────────── GradientScale ───────────────────────────────

/// A reading on a scale that runs from bad to good (signal strength, density, latency): a
/// graded rail, a handle at the value and numbered ticks. The value glides when it changes.
#[derive(IntoElement)]
pub struct GradientScale {
    follow: dodai_gpui::follow::Follow,
    range: (f32, f32),
    value: f32,
    good_high: bool,
    fmt: Option<NumFmt>,
}

impl GradientScale {
    dodai_gpui::follow_api!();
    /// `value` on a scale from `lo` (bad) to `hi` (good).
    #[track_caller]
    pub fn new(lo: f32, hi: f32, value: f32) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            range: (lo, hi),
            value,
            good_high: true,
            fmt: None,
        }
    }
    /// Low values are the good ones (latency, error rate).
    pub fn low_is_good(mut self, on: bool) -> Self {
        self.good_high = !on;
        self
    }
    /// Custom formatter for the tick labels.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.fmt = Some(Rc::new(f));
        self
    }

    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (lo, hi) = match self.range {
            (lo, hi) if lo.is_finite() && hi.is_finite() && lo != hi => (lo, hi),
            _ => (0.0, 1.0),
        };
        let value = if self.value.is_finite() {
            self.value.clamp(lo.min(hi), lo.max(hi))
        } else {
            lo
        };
        let v = dodai_gpui::follow::follow_all(window, cx, &self.follow, &[value])
            .first()
            .copied()
            .unwrap_or(value);
        let ticks = nice_ticks(lo.min(hi), lo.max(hi), 12);
        let at = norm(v, lo, hi);
        let rail = t.spacing.xs * 1.5;
        let knob = t.sizes().slider_thumb;
        let stop = |f: f32| {
            let f = if self.good_high { f } else { 1.0 - f };
            hsla(bad_to_good(&t, f).mix(t.palette.surface, 0.55))
        };
        let half = |a: f32, b: f32| {
            div().flex_1().h_full().bg(gpui::linear_gradient(
                90.0,
                gpui::linear_color_stop(stop(a), 0.0),
                gpui::linear_color_stop(stop(b), 1.0),
            ))
        };
        let tone = hsla(bad_to_good(&t, if self.good_high { at } else { 1.0 - at }));
        let track = div()
            .relative()
            .w_full()
            .h(px(knob))
            .flex()
            .items_center()
            .child(
                div()
                    .w_full()
                    .h(px(rail))
                    .rounded(px(t.shape.radius.min(rail / 2.0)))
                    .overflow_hidden()
                    .flex()
                    .child(half(0.0, 0.5))
                    .child(half(0.5, 1.0)),
            )
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left(relative(at))
                    .ml(px(-knob / 2.0))
                    .size(px(knob))
                    .rounded_full()
                    .bg(hsla(t.palette.surface_raised))
                    .border_2()
                    .border_color(tone),
            );
        let labels = label_style(&t, div())
            .relative()
            .w_full()
            .h(px(t.typography.size_xs * 1.4))
            .children(ticks.iter().map(|tk| {
                div()
                    .absolute()
                    .left(relative(norm(*tk, lo, hi)))
                    .w(px(0.0))
                    .flex()
                    .justify_center()
                    .whitespace_nowrap()
                    .child(fmt_with(&self.fmt, *tk))
            }));
        let focus = Focus::new(window, cx, &self.follow, 1);
        let text = focus.shown(cx).map(|_| {
            let along = norm(value, lo, hi) * 100.0;
            let along = if self.good_high { along } else { 100.0 - along };
            format!(
                "{} · {along:.0}% of the way to good",
                fmt_with(&self.fmt, value)
            )
        });
        let key = SharedString::from(format!(
            "scale-{:?}",
            self.follow.sub("scale").element_id(window, cx)
        ));
        let body = div()
            .id(key)
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .px(px(knob / 2.0))
            .child(track)
            .child(labels);
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(focus.bind(body, 0))
            .child(readout(&t, text, "Hover the scale"))
    }
}

dodai_gpui::entry_render!(GradientScale);
